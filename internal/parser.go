package internal

import (
	"errors"
	"fmt"
)

type callStack struct {
	function  string
	loopCount int
}

// parser stores parser data
type parser[T any] struct {
	current int

	cls []*callStack

	state *interpreterState[T]
}

func (p *parser[T]) getParsingContext() *callStack {
	return p.cls[len(p.cls)-1]
}

func (p *parser[T]) enterFunction(name string) {
	p.cls = append(p.cls, &callStack{
		function:  name,
		loopCount: 0,
	})
}

func (p *parser[T]) leaveFunction(name string) {
	pc := p.getParsingContext()
	if pc.function != name {
		p.state.fatalError(errMaxParameters, p.peek().line, 0)
	}
	p.cls = p.cls[:len(p.cls)-1]
}

func (p *parser[T]) enterLoop() {
	pc := p.getParsingContext()
	pc.loopCount++
}

func (p *parser[T]) leaveLoop() {
	pc := p.getParsingContext()
	pc.loopCount--
}

func (p *parser[T]) insideLoop() bool {
	return p.getParsingContext().loopCount != 0
}

const maxFunctionParams = 255

func (p *parser[T]) parse() {
	p.cls = make([]*callStack, 0)
	p.enterFunction("")
	defer p.leaveFunction("")
	for !p.isAtEnd() {
		st := p.parseStmt()
		// When multiple empty lines are encountered after a statement
		// the parser founds nil statements, we should avoid them to not
		// break the execution stage
		if st != nil {
			p.state.stmts = append(p.state.stmts, st)
		}
	}
}

func (p *parser[T]) parseStmt() stmt[T] {
	defer func() {
		if r := recover(); r != nil {
			p.synchronize()
		}
	}()
	return p.declaration(true)
}

func (p *parser[T]) declaration(expectNewLine bool) stmt[T] {
	var s stmt[T]
	if p.match(tkClass) {
		s = p.class()
	} else if p.match(tkFn) {
		s = p.fn()
	} else if p.match(tkLet) {
		s = p.let()
	} else {
		s = p.statement()
	}
	if expectNewLine {
		p.consume(tkNewline, errExpectedNewline)
	}
	return s
}

func (p *parser[T]) class() stmt[T] {
	name := p.consume(tkIdentifier, errExpectedIdentifier)

	var superclass *variableExpr[T]
	if p.match(tkLess) {
		class := p.consume(tkIdentifier, errExpectedIdentifier)
		superclass = &variableExpr[T]{
			name: class,
		}
	}

	p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)

	var methods []*fnStmt[T]
	var staticMethods []*fnStmt[T]
	for !p.check(tkRightCurlyBrace) && !p.isAtEnd() {
		if p.match(tkClass) {
			staticMethods = append(staticMethods, p.fn())
		} else {
			methods = append(methods, p.fn())
		}
	}

	p.consume(tkRightCurlyBrace, errExpectedClosingCurlyBrace)

	return &classStmt[T]{
		name:          name,
		methods:       methods,
		staticMethods: staticMethods,
		superclass:    superclass,
	}
}

func (p *parser[T]) fn() *fnStmt[T] {
	name := p.consume(tkIdentifier, errExpectedFunctionName)

	p.enterFunction(name.lexeme)
	defer p.leaveFunction(name.lexeme)

	p.consume(tkLeftParen, errExpectedParen)

	var params []*token
	if !p.check(tkRightParen) {
		for {
			if len(params) > maxFunctionParams {
				p.state.fatalError(errMaxParameters, p.peek().line, 0)
			}
			params = append(params, p.consume(tkIdentifier, errExpectedFunctionParam))
			if !p.match(tkComma) {
				break
			}
		}
	}
	p.consume(tkRightParen, errUnclosedParen)

	body := make([]stmt[T], 0)
	if p.match(tkLeftCurlyBrace) {
		body = p.block()
	} else {
		body = append(body, p.expressionStmt())
	}

	return &fnStmt[T]{
		name:   name,
		params: params,
		body:   body,
	}
}

func (p *parser[T]) fnExpr() *functionExpr[T] {
	p.consume(tkLeftParen, errExpectedParen)

	lambdaName := fmt.Sprintf("lambda%d", len(p.cls))
	p.enterFunction(lambdaName)
	defer p.leaveFunction(lambdaName)

	var params []*token
	if !p.check(tkRightParen) {
		for {
			if len(params) > maxFunctionParams {
				p.state.fatalError(errMaxParameters, p.peek().line, 0)
			}
			params = append(params, p.consume(tkIdentifier, errExpectedFunctionParam))
			if !p.match(tkComma) {
				break
			}
		}
	}
	p.consume(tkRightParen, errUnclosedParen)

	body := make([]stmt[T], 0)
	if p.match(tkLeftCurlyBrace) {
		body = p.block()
	} else {
		body = append(body, p.expressionStmt())
	}

	return &functionExpr[T]{
		params: params,
		body:   body,
	}
}

func (p *parser[T]) let() stmt[T] {
	name := p.consume(tkIdentifier, errExpectedIdentifier)

	var init expr[T]
	if p.match(tkEqual) {
		init = p.expression()
	}

	return &letStmt[T]{
		name:        name,
		initializer: init,
	}
}

func (p *parser[T]) statement() stmt[T] {
	if p.match(tkFor) {
		return p.forLoop()
	}
	if p.match(tkTry) {
		return p.tryCatch()
	}
	if p.match(tkIf) {
		return p.ifStmt()
	}
	if p.match(tkReturn) {
		return p.ret()
	}
	if p.match(tkBreak) {
		return p.brk()
	}
	if p.match(tkContinue) {
		return p.cont()
	}
	if p.match(tkWhile) {
		return p.while()
	}
	if p.match(tkLeftCurlyBrace) {
		return &blockStmt[T]{stmts: p.block()}
	}
	return p.expressionStmt()
}

func (p *parser[T]) forLoop() stmt[T] {
	keyword := p.previous()

	p.enterLoop()
	defer p.leaveLoop()

	if p.check(tkIdentifier) {
		// Enhanced for
		return p.enhancedFor(keyword)
	}
	// Classic for
	var init stmt[T]
	if p.match(tkSemicolon) {
		init = nil
	} else if p.match(tkLet) {
		init = p.let()
		p.consume(tkSemicolon, errExpectedSemicolon)
	} else {
		p.state.setError(errExpectedInit, p.peek().line, 0)
	}

	cond := p.expression()
	p.consume(tkSemicolon, errExpectedSemicolon)

	inc := p.expression()

	body := p.declaration(false)

	return &classicForStmt[T]{
		keyword:     keyword,
		initializer: init,
		condition:   cond,
		increment:   inc,
		body:        body,
	}
}

func (p *parser[T]) enhancedFor(keyword *token) stmt[T] {
	var ids []*token
	for p.match(tkIdentifier) {
		ids = append(ids, p.previous())
		p.match(tkComma)
	}
	p.consume(tkIn, errExpectedIn)
	collection := p.expression()
	body := p.declaration(false)
	return &enhancedForStmt[T]{
		keyword:     keyword,
		identifiers: ids,
		body:        body,
		collection:  collection,
	}
}

func (p *parser[T]) tryCatch() stmt[T] {
	tryBody := p.declaration(false)
	p.consume(tkCatch, errExpectedCatch)
	name := p.consume(tkIdentifier, errExpectedIdentifier)
	catchBody := p.declaration(false)

	return &tryCatchStmt[T]{
		tryBody:   tryBody,
		name:      name,
		catchBody: catchBody,
	}
}

func (p *parser[T]) ifStmt() stmt[T] {
	st := &ifStmt[T]{
		keyword: p.previous(),
	}

	st.condition = p.expression()

	p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)

	for !p.match(tkRightCurlyBrace) {
		st.thenBranch = append(st.thenBranch, p.declaration(false))
	}

	for p.match(tkElif) {
		elif := &struct {
			condition  expr[T]
			thenBranch []stmt[T]
		}{
			condition: p.expression(),
		}
		p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)
		for !p.match(tkRightCurlyBrace) {
			elif.thenBranch = append(elif.thenBranch, p.declaration(false))
		}
		st.elifs = append(st.elifs, elif)
	}

	if p.match(tkElse) {
		p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)
		for !p.check(tkRightCurlyBrace) {
			st.elseBranch = append(st.elseBranch, p.declaration(false))
		}
		p.consume(tkRightCurlyBrace, errExpectedClosingCurlyBrace)
	}

	return st
}

func (p *parser[T]) ret() stmt[T] {
	var value expr[T]
	keyword := p.previous()
	if !p.check(tkNewline) {
		value = p.expression()
	}
	return &returnStmt[T]{
		keyword: keyword,
		value:   value,
	}
}

func (p *parser[T]) brk() stmt[T] {
	keyword := p.previous()
	if !p.insideLoop() {
		p.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
	}
	return &breakStmt[T]{
		keyword: keyword,
	}
}

func (p *parser[T]) cont() stmt[T] {
	keyword := p.previous()
	if !p.insideLoop() {
		p.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
	}
	return &continueStmt[T]{
		keyword: keyword,
	}
}

func (p *parser[T]) while() stmt[T] {
	keyword := p.previous()
	p.enterLoop()
	defer p.leaveLoop()
	cond := p.expression()
	body := p.declaration(false)
	return &whileStmt[T]{
		keyword:   keyword,
		condition: cond,
		body:      body,
	}
}

func (p *parser[T]) block() []stmt[T] {
	var stmts []stmt[T]
	for !p.match(tkRightCurlyBrace) {
		stmts = append(stmts, p.declaration(false))
	}
	return stmts
}

func (p *parser[T]) expressionStmt() stmt[T] {
	expr := p.expression()
	if expr != nil {
		return &exprStmt[T]{
			last:       p.previous(),
			expression: expr,
		}
	}
	// expr is nil when there are multiple empty lines
	return nil
}

func (p *parser[T]) expression() expr[T] {
	return p.assignment()
}

func (p *parser[T]) list() expr[T] {
	elements := p.arguments(tkRightBrace)
	brace := p.consume(tkRightBrace, errUnclosedBracket)
	return &listExpr[T]{
		elements: elements,
		brace:    brace,
	}
}

func (p *parser[T]) dictionary() expr[T] {
	elements := p.dictElements()
	curlyBrace := p.consume(tkRightCurlyBrace, errUnclosedCurlyBrace)
	return &dictionaryExpr[T]{
		elements:   elements,
		curlyBrace: curlyBrace,
	}
}

// dictElements returns array of keys & values where keys
// are stored in even positions and values in odd positions
func (p *parser[T]) dictElements() []expr[T] {
	elements := make([]expr[T], 0)
	for !p.check(tkRightCurlyBrace) {
		key := p.expression()
		p.consume(tkColon, errExpectedColon)
		value := p.expression()
		elements = append(elements, key, value)
		if !p.match(tkComma) {
			break
		}
	}
	return elements
}

func (p *parser[T]) assignment() expr[T] {
	expr := p.or()
	if p.match(tkEqual) {
		equal := p.previous()
		value := p.assignment()

		access, isAccess := expr.(*accessExpr[T])
		if isAccess {
			object := access.object
			for {
				_, ok := object.(*accessExpr[T])
				if !ok {
					break
				}
				object = object.(*accessExpr[T]).object
			}
			expr = object
		}

		if variable, isVar := expr.(*variableExpr[T]); isVar {
			assign := &assignExpr[T]{
				name:  variable.name,
				value: value,
			}
			if access != nil {
				assign.access = access
			}
			return assign
		} else if get, isGet := expr.(*getExpr[T]); isGet {
			set := &setExpr[T]{
				name:   get.name,
				object: get.object,
				value:  value,
			}
			if access != nil {
				set.access = access
			}
			return set
		}

		p.state.fatalError(errUndefinedStmt, equal.line, 0)
	}
	return expr
}

func (p *parser[T]) access(object expr[T]) expr[T] {
	slice := &accessExpr[T]{
		object: object,
		brace:  p.previous(),
	}
	p.slice(slice)
	p.consume(tkRightBrace, errors.New("Expected ']' at the end of slice"))
	return slice
}

func (p *parser[T]) slice(slice *accessExpr[T]) {
	if p.match(tkColon) {
		slice.firstColon = p.previous()
		if p.match(tkColon) {
			slice.secondColon = p.previous()
			slice.third = p.expression()
		} else {
			slice.second = p.expression()
			if p.match(tkColon) {
				slice.secondColon = p.previous()
				slice.third = p.expression()
			}
		}
	} else {
		slice.first = p.expression()
		if p.match(tkColon) {
			slice.firstColon = p.previous()
			if p.match(tkColon) {
				slice.secondColon = p.previous()
				slice.third = p.expression()
			} else if !p.check(tkRightBrace) && !p.isAtEnd() {
				slice.second = p.expression()
				if p.match(tkColon) {
					slice.secondColon = p.previous()
					if !p.check(tkRightBrace) && !p.isAtEnd() {
						slice.third = p.expression()
					}
				}
			}
		}
	}
}

func (p *parser[T]) or() expr[T] {
	expr := p.and()
	for p.match(tkOr) {
		operator := p.previous()
		right := p.and()
		expr = &logicalExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) and() expr[T] {
	expr := p.equality()
	for p.match(tkAnd) {
		operator := p.previous()
		right := p.equality()
		expr = &logicalExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) equality() expr[T] {
	expr := p.comparison()
	for p.match(tkEqualEqual, tkBangEqual) {
		operator := p.previous()
		right := p.comparison()
		expr = &binaryExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) comparison() expr[T] {
	expr := p.addition()
	for p.match(tkGreater, tkGreaterEqual, tkLess, tkLessEqual) {
		operator := p.previous()
		right := p.addition()
		expr = &binaryExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) addition() expr[T] {
	expr := p.multiplication()
	for p.match(tkPlus, tkMinus) {
		operator := p.previous()
		right := p.multiplication()
		expr = &binaryExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) multiplication() expr[T] {
	expr := p.power()
	for p.match(tkSlash, tkMod, tkStar) {
		operator := p.previous()
		right := p.power()
		expr = &binaryExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) power() expr[T] {
	expr := p.unary()
	for p.match(tkPower) {
		operator := p.previous()
		right := p.unary()
		expr = &binaryExpr[T]{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser[T]) unary() expr[T] {
	if p.match(tkNot, tkMinus) {
		operator := p.previous()
		right := p.unary()
		return &unaryExpr[T]{
			operator: operator,
			right:    right,
		}
	}
	return p.call()
}

func (p *parser[T]) call() expr[T] {
	expr := p.primary()
	for {
		if p.match(tkLeftParen) {
			expr = p.finishCall(expr)
		} else if p.match(tkDot) {
			name := p.consume(tkIdentifier, errExpectedProp)
			expr = &getExpr[T]{
				object: expr,
				name:   name,
			}
		} else if p.match(tkLeftBrace) {
			expr = p.access(expr)
		} else {
			break
		}
	}
	return expr
}

func (p *parser[T]) finishCall(callee expr[T]) expr[T] {
	arguments := p.arguments(tkRightParen)
	paren := p.consume(tkRightParen, errors.New("Expect ')' after arguments"))
	return &callExpr[T]{
		callee:    callee,
		arguments: arguments,
		paren:     paren,
	}
}

func (p *parser[T]) arguments(tk tokenType) []expr[T] {
	arguments := make([]expr[T], 0)
	if !p.check(tk) {
		for {
			if tk == tkRightParen && len(arguments) >= maxFunctionParams {
				p.state.fatalError(errMaxArguments, p.peek().line, 0)
			}
			arguments = append(arguments, p.expression())
			if !p.match(tkComma) || p.check(tk) {
				break
			}
		}
	}
	return arguments
}

func (p *parser[T]) primary() expr[T] {
	if p.match(tkNumber, tkString) {
		return &literalExpr[T]{value: p.previous().literal}
	}
	if p.match(tkFalse) {
		return &literalExpr[T]{value: grotskyBool(false)}
	}
	if p.match(tkTrue) {
		return &literalExpr[T]{value: grotskyBool(true)}
	}
	if p.match(tkNil) {
		return &literalExpr[T]{value: nil}
	}
	if p.match(tkIdentifier) {
		return &variableExpr[T]{name: p.previous()}
	}
	if p.match(tkLeftParen) {
		expr := p.expression()
		p.consume(tkRightParen, errUnclosedParen)
		return &groupingExpr[T]{expression: expr}
	}
	if p.match(tkLeftBrace) {
		return p.list()
	}
	if p.match(tkLeftCurlyBrace) {
		return p.dictionary()
	}
	if p.match(tkFn) {
		return p.fnExpr()
	}
	if p.match(tkThis) {
		return &thisExpr[T]{keyword: p.previous()}
	}
	if p.match(tkSuper) {
		return p.superExpr()
	}
	if p.match(tkNewline) {
		return nil
	}

	p.state.fatalError(errUndefinedExpr, p.peek().line, 0)
	return &literalExpr[T]{}
}

func (p *parser[T]) superExpr() expr[T] {
	super := &superExpr[T]{
		keyword: p.previous(),
	}
	if !p.check(tkLeftParen) {
		p.consume(tkDot, errExpectedDot)
		super.method = p.consume(tkIdentifier, errExpectedIdentifier)
	} else {
		super.method = &token{
			token:  tkIdentifier,
			lexeme: "init",
			line:   super.keyword.line,
		}
	}
	return super
}

func (p *parser[T]) consume(tk tokenType, err error) *token {
	if p.check(tk) {
		return p.advance()
	}

	p.state.setError(err, p.peek().line, 0)
	return &token{}
}

func (p *parser[T]) advance() *token {
	if !p.isAtEnd() {
		p.current++
	}
	return p.previous()
}

func (p *parser[T]) match(tokens ...tokenType) bool {
	for _, token := range tokens {
		if p.check(token) {
			p.current++
			return true
		}
	}
	return false
}

func (p *parser[T]) check(token tokenType) bool {
	oldCurrent := p.current
	for token != tkNewline && !p.isAtEnd() && p.peek().token == tkNewline {
		p.current++
	}
	matchs := p.peek().token == token
	if !matchs {
		p.current = oldCurrent
	}
	return matchs
}

func (p *parser[T]) peek() token {
	return p.state.tokens[p.current]
}

func (p *parser[T]) previous() *token {
	for i := 1; i <= p.current; i-- {
		if p.state.tokens[p.current-i].token != tkNewline {
			break
		}
	}
	return &p.state.tokens[p.current-1]
}

func (p *parser[T]) isAtEnd() bool {
	return p.peek().token == tkEOF
}

func (p *parser[T]) synchronize() {
	p.advance()
	for !p.isAtEnd() {
		switch p.peek().token {
		case tkClass:
			return
		case tkFn:
			return
		case tkLet:
			return
		case tkFor:
			return
		case tkIf:
			return
		case tkWhile:
			return
		case tkReturn:
			return
		default:
		}

		p.advance()
	}
}
