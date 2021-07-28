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
type parser struct {
	current int

	cls []*callStack

	state *interpreterState
}

func (p *parser) getParsingContext() *callStack {
	return p.cls[len(p.cls)-1]
}

func (p *parser) enterFunction(name string) {
	p.cls = append(p.cls, &callStack{
		function:  name,
		loopCount: 0,
	})
}

func (p *parser) leaveFunction(name string) {
	pc := p.getParsingContext()
	if pc.function != name {
		p.state.fatalError(errMaxParameters, p.peek().line, 0)
	}
	p.cls = p.cls[:len(p.cls)-1]
}

func (p *parser) enterLoop() {
	pc := p.getParsingContext()
	pc.loopCount++
}

func (p *parser) leaveLoop() {
	pc := p.getParsingContext()
	pc.loopCount--
}

func (p *parser) insideLoop() bool {
	return p.getParsingContext().loopCount != 0
}

const maxFunctionParams = 255

func (p *parser) parse() {
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

func (p *parser) parseStmt() stmt {
	defer func() {
		if r := recover(); r != nil {
			p.synchronize()
		}
	}()
	return p.declaration(true)
}

func (p *parser) declaration(expectNewLine bool) stmt {
	var s stmt
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

func (p *parser) class() stmt {
	name := p.consume(tkIdentifier, errExpectedIdentifier)

	var superclass *variableExpr
	if p.match(tkLess) {
		class := p.consume(tkIdentifier, errExpectedIdentifier)
		superclass = &variableExpr{
			name: class,
		}
	}

	p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)

	var methods []*fnStmt
	var staticMethods []*fnStmt
	for !p.check(tkRightCurlyBrace) && !p.isAtEnd() {
		if p.match(tkClass) {
			staticMethods = append(staticMethods, p.fn())
		} else {
			methods = append(methods, p.fn())
		}
	}

	p.consume(tkRightCurlyBrace, errExpectedClosingCurlyBrace)

	return &classStmt{
		name:          name,
		methods:       methods,
		staticMethods: staticMethods,
		superclass:    superclass,
	}
}

func (p *parser) fn() *fnStmt {
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

	body := make([]stmt, 0)
	if p.match(tkLeftCurlyBrace) {
		body = p.block()
	} else {
		body = append(body, p.expressionStmt())
	}

	return &fnStmt{
		name:   name,
		params: params,
		body:   body,
	}
}

func (p *parser) fnExpr() *functionExpr {
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

	body := make([]stmt, 0)
	if p.match(tkLeftCurlyBrace) {
		body = p.block()
	} else {
		body = append(body, p.expressionStmt())
	}

	return &functionExpr{
		params: params,
		body:   body,
	}
}

func (p *parser) let() stmt {
	name := p.consume(tkIdentifier, errExpectedIdentifier)

	var init expr
	if p.match(tkEqual) {
		init = p.expression()
	}

	return &letStmt{
		name:        name,
		initializer: init,
	}
}

func (p *parser) statement() stmt {
	if p.match(tkFor) {
		return p.forLoop()
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
		return &blockStmt{stmts: p.block()}
	}
	return p.expressionStmt()
}

func (p *parser) forLoop() stmt {
	keyword := p.previous()

	p.enterLoop()
	defer p.leaveLoop()

	if p.check(tkIdentifier) {
		// Enhanced for
		return p.enhancedFor(keyword)
	}
	// Classic for
	var init stmt
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

	return &classicForStmt{
		keyword:     keyword,
		initializer: init,
		condition:   cond,
		increment:   inc,
		body:        body,
	}
}

func (p *parser) enhancedFor(keyword *token) stmt {
	var ids []*token
	for p.match(tkIdentifier) {
		ids = append(ids, p.previous())
		p.match(tkComma)
	}
	p.consume(tkIn, errExpectedIn)
	collection := p.expression()
	body := p.declaration(false)
	return &enhancedForStmt{
		keyword:     keyword,
		identifiers: ids,
		body:        body,
		collection:  collection,
	}
}

func (p *parser) ifStmt() stmt {
	st := &ifStmt{
		keyword: p.previous(),
	}

	st.condition = p.expression()

	p.consume(tkLeftCurlyBrace, errExpectedOpeningCurlyBrace)

	for !p.match(tkRightCurlyBrace) {
		st.thenBranch = append(st.thenBranch, p.declaration(false))
	}

	for p.match(tkElif) {
		elif := &struct {
			condition  expr
			thenBranch []stmt
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

func (p *parser) ret() stmt {
	var value expr
	keyword := p.previous()
	if !p.check(tkNewline) {
		value = p.expression()
	}
	return &returnStmt{
		keyword: keyword,
		value:   value,
	}
}

func (p *parser) brk() stmt {
	keyword := p.previous()
	if !p.insideLoop() {
		p.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
	}
	return &breakStmt{
		keyword: keyword,
	}
}

func (p *parser) cont() stmt {
	keyword := p.previous()
	if !p.insideLoop() {
		p.state.setError(errOnlyAllowedInsideLoop, keyword.line, 0)
	}
	return &continueStmt{
		keyword: keyword,
	}
}

func (p *parser) while() stmt {
	keyword := p.previous()
	p.enterLoop()
	defer p.leaveLoop()
	cond := p.expression()
	body := p.declaration(false)
	return &whileStmt{
		keyword:   keyword,
		condition: cond,
		body:      body,
	}
}

func (p *parser) block() []stmt {
	var stmts []stmt
	for !p.match(tkRightCurlyBrace) {
		stmts = append(stmts, p.declaration(false))
	}
	return stmts
}

func (p *parser) expressionStmt() stmt {
	expr := p.expression()
	if expr != nil {
		return &exprStmt{
			last:       p.previous(),
			expression: expr,
		}
	}
	// expr is nil when there are multiple empty lines
	return nil
}

func (p *parser) expression() expr {
	return p.assignment()
}

func (p *parser) list() expr {
	elements := p.arguments(tkRightBrace)
	brace := p.consume(tkRightBrace, errUnclosedBracket)
	return &listExpr{
		elements: elements,
		brace:    brace,
	}
}

func (p *parser) dictionary() expr {
	elements := p.dictElements()
	curlyBrace := p.consume(tkRightCurlyBrace, errUnclosedCurlyBrace)
	return &dictionaryExpr{
		elements:   elements,
		curlyBrace: curlyBrace,
	}
}

// dictElements returns array of keys & values where keys
// are stored in even positions and values in odd positions
func (p *parser) dictElements() []expr {
	elements := make([]expr, 0)
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

func (p *parser) assignment() expr {
	expr := p.or()
	if p.match(tkEqual) {
		equal := p.previous()
		value := p.assignment()

		access := expr
		for p.match(tkLeftBrace) {
			access = p.access(access)
		}

		if variable, isVar := expr.(*variableExpr); isVar {
			assign := &assignExpr{
				name:  variable.name,
				value: value,
			}
			if expr != access {
				assign.access = access
			}
			return assign
		} else if get, isGet := expr.(*getExpr); isGet {
			set := &setExpr{
				name:   get.name,
				object: get.object,
				value:  value,
			}
			if expr != access {
				set.access = access
			}
			return set
		}

		p.state.fatalError(errUndefinedStmt, equal.line, 0)
	}
	return expr
}

func (p *parser) access(object expr) expr {
	slice := &accessExpr{
		object: object,
		brace:  p.previous(),
	}
	p.slice(slice)
	p.consume(tkRightBrace, errors.New("Expected ']' at the end of slice"))
	return slice
}

func (p *parser) slice(slice *accessExpr) {
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

func (p *parser) or() expr {
	expr := p.and()
	for p.match(tkOr) {
		operator := p.previous()
		right := p.and()
		expr = &logicalExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) and() expr {
	expr := p.equality()
	for p.match(tkAnd) {
		operator := p.previous()
		right := p.equality()
		expr = &logicalExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) equality() expr {
	expr := p.comparison()
	for p.match(tkEqualEqual, tkBangEqual) {
		operator := p.previous()
		right := p.comparison()
		expr = &binaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) comparison() expr {
	expr := p.addition()
	for p.match(tkGreater, tkGreaterEqual, tkLess, tkLessEqual) {
		operator := p.previous()
		right := p.addition()
		expr = &binaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) addition() expr {
	expr := p.multiplication()
	for p.match(tkPlus, tkMinus) {
		operator := p.previous()
		right := p.multiplication()
		expr = &binaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) multiplication() expr {
	expr := p.power()
	for p.match(tkSlash, tkMod, tkStar) {
		operator := p.previous()
		right := p.power()
		expr = &binaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) power() expr {
	expr := p.unary()
	for p.match(tkPower) {
		operator := p.previous()
		right := p.unary()
		expr = &binaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *parser) unary() expr {
	if p.match(tkNot, tkMinus) {
		operator := p.previous()
		right := p.unary()
		return &unaryExpr{
			operator: operator,
			right:    right,
		}
	}
	return p.call()
}

func (p *parser) call() expr {
	expr := p.primary()
	for {
		if p.match(tkLeftParen) {
			expr = p.finishCall(expr)
		} else if p.match(tkDot) {
			name := p.consume(tkIdentifier, errExpectedProp)
			expr = &getExpr{
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

func (p *parser) finishCall(callee expr) expr {
	arguments := p.arguments(tkRightParen)
	paren := p.consume(tkRightParen, errors.New("Expect ')' after arguments"))
	return &callExpr{
		callee:    callee,
		arguments: arguments,
		paren:     paren,
	}
}

func (p *parser) arguments(tk tokenType) []expr {
	arguments := make([]expr, 0)
	if !p.check(tk) {
		for {
			if tk == tkRightParen && len(arguments) >= maxFunctionParams {
				p.state.fatalError(errMaxArguments, p.peek().line, 0)
			}
			arguments = append(arguments, p.expression())
			if !p.match(tkComma) {
				break
			}
		}
	}
	return arguments
}

func (p *parser) primary() expr {
	if p.match(tkNumber, tkString) {
		return &literalExpr{value: p.previous().literal}
	}
	if p.match(tkFalse) {
		return &literalExpr{value: grotskyBool(false)}
	}
	if p.match(tkTrue) {
		return &literalExpr{value: grotskyBool(true)}
	}
	if p.match(tkNil) {
		return &literalExpr{value: nil}
	}
	if p.match(tkIdentifier) {
		return &variableExpr{name: p.previous()}
	}
	if p.match(tkLeftParen) {
		expr := p.expression()
		p.consume(tkRightParen, errUnclosedParen)
		return &groupingExpr{expression: expr}
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
		return &thisExpr{keyword: p.previous()}
	}
	if p.match(tkSuper) {
		return p.superExpr()
	}
	if p.match(tkNewline) {
		return nil
	}

	p.state.fatalError(errUndefinedExpr, p.peek().line, 0)
	return &literalExpr{}
}

func (p *parser) superExpr() expr {
	super := &superExpr{
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

func (p *parser) consume(tk tokenType, err error) *token {
	if p.check(tk) {
		return p.advance()
	}

	p.state.setError(err, p.peek().line, 0)
	return &token{}
}

func (p *parser) advance() *token {
	if !p.isAtEnd() {
		p.current++
	}
	return p.previous()
}

func (p *parser) match(tokens ...tokenType) bool {
	for _, token := range tokens {
		if p.check(token) {
			p.current++
			return true
		}
	}
	return false
}

func (p *parser) check(token tokenType) bool {
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

func (p *parser) peek() token {
	return p.state.tokens[p.current]
}

func (p *parser) previous() *token {
	for i := 1; i <= p.current; i-- {
		if p.state.tokens[p.current-i].token != tkNewline {
			break
		}
	}
	return &p.state.tokens[p.current-1]
}

func (p *parser) isAtEnd() bool {
	return p.peek().token == tkEOF
}

func (p *parser) synchronize() {
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
