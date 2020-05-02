package internal

import (
	"errors"
)

// parser stores parser data
type parser struct {
	state   *state
	current int
}

const maxFunctionParams = 255

func (p *parser) parse() {
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
	return p.declaration()
}

func (p *parser) declaration() stmt {
	defer p.consume(NEWLINE, errExpectedNewline)
	if p.match(CLASS) {
		return p.class()
	}
	if p.match(FN) {
		return p.fn()
	}
	if p.match(LET) {
		return p.let()
	}
	return p.statement()
}

func (p *parser) class() stmt {
	//TODO: implement
	return nil
}

func (p *parser) fn() stmt {
	name := p.consume(IDENTIFIER, errExpectedFunctionName)
	p.consume(LEFT_PAREN, errExpectedParen)

	var params []*token
	if !p.check(RIGHT_PAREN) {
		for {
			if len(params) > maxFunctionParams {
				p.state.fatalError(errMaxParameters, p.peek().line, 0)
			}
			params = append(params, p.consume(IDENTIFIER, errExpectedFunctionParam))
			if !p.match(COMMA) {
				break
			}
		}
	}
	p.consume(RIGHT_PAREN, errUnclosedParen)

	p.consume(BEGIN, errExpectedBegin)

	body := p.block()

	return &fnStmt{
		name:   name,
		params: params,
		body:   body,
	}
}

func (p *parser) let() stmt {
	name := p.consume(IDENTIFIER, errExpectedIdentifier)

	var init expr
	if p.match(EQUAL) {
		init = p.expression()
	}

	return &letStmt{
		name:        name,
		initializer: init,
	}
}

func (p *parser) statement() stmt {
	if p.match(FOR) {
		return p.forLoop()
	}
	if p.match(IF) {
		return p.ifStmt()
	}
	if p.match(RETURN) {
		return p.ret()
	}
	if p.match(WHILE) {
		return p.while()
	}
	if p.match(BEGIN) {
		return &blockStmt{stmts: p.block()}
	}
	return p.expressionStmt()
}

func (p *parser) forLoop() stmt {
	if p.check(IDENTIFIER) {
		// Enhanced for
		return p.enhancedFor()
	}
	// Classic for
	var init stmt
	if p.match(COMMA) {
		init = nil
	} else if p.match(LET) {
		init = p.let()
		p.consume(COMMA, errExpectedComma)
	} else {
		init = p.expressionStmt()
		p.consume(COMMA, errExpectedComma)
	}

	cond := p.expression()
	p.consume(COMMA, errExpectedComma)

	inc := p.expression()

	body := p.statement()

	return &classicForStmt{
		initializer: init,
		condition:   cond,
		increment:   inc,
		body:        body,
	}
}

func (p *parser) enhancedFor() stmt {
	var ids []*token
	for p.match(IDENTIFIER) {
		ids = append(ids, p.previous())
		p.match(COMMA)
	}
	p.consume(IN, errExpectedIn)
	collection := p.expression()
	body := p.statement()
	return &enhancedForStmt{
		identifiers: ids,
		body:        body,
		collection:  collection,
	}
}

func (p *parser) ifStmt() stmt {
	st := &ifStmt{}

	st.condition = p.expression()

	p.consume(BEGIN, errExpectedBegin)

	for !p.check(ELIF) && !p.check(ELSE) && !p.check(END) {
		st.thenBranch = append(st.thenBranch, p.statement())
	}

	for p.match(ELIF) {
		elif := &struct {
			condition  expr
			thenBranch []stmt
		}{
			condition: p.expression(),
		}
		for !p.check(ELIF) && !p.check(ELSE) && !p.check(END) {
			elif.thenBranch = append(elif.thenBranch, p.statement())
		}
		st.elifs = append(st.elifs, elif)
	}

	if p.match(ELSE) {
		for !p.check(END) {
			st.elseBranch = append(st.elseBranch, p.statement())
		}
	}

	p.consume(END, errExpectedEnd)

	return st
}

func (p *parser) ret() stmt {
	var value expr
	keyword := p.previous()
	if !p.check(NEWLINE) {
		value = p.expression()
	}
	return &returnStmt{
		keyword: keyword,
		value:   value,
	}
}

func (p *parser) while() stmt {
	cond := p.expression()
	body := p.statement()
	return &whileStmt{
		condition: cond,
		body:      body,
	}
}

func (p *parser) block() []stmt {
	var stmts []stmt
	p.consume(NEWLINE, errExpectedNewline)
	for !p.match(END) {
		stmts = append(stmts, p.declaration())
	}
	return stmts
}

func (p *parser) expressionStmt() stmt {
	expr := p.expression()
	return &exprStmt{expression: expr}
}

func (p *parser) expression() expr {
	return p.assignment()
}

func (p *parser) list() expr {
	elements := p.arguments(RIGHT_BRACE)
	brace := p.consume(RIGHT_BRACE, errUnclosedBracket)
	return &listExpr{
		elements: elements,
		brace:    brace,
	}
}

func (p *parser) dictionary() expr {
	elements := p.dictElements()
	curlyBrace := p.consume(RIGHT_CURLY_BRACE, errUnclosedCurlyBrace)
	return &dictionaryExpr{
		elements:   elements,
		curlyBrace: curlyBrace,
	}
}

// dictElements returns array of keys & values where keys
// are stored in even positions and values in odd positions
func (p *parser) dictElements() []expr {
	elements := make([]expr, 0)
	for !p.check(RIGHT_CURLY_BRACE) {
		key := p.expression()
		p.consume(COLON, errExpectedColon)
		value := p.expression()
		elements = append(elements, key, value)
		if !p.match(COMMA) {
			break
		}
	}
	return elements
}

func (p *parser) assignment() expr {
	expr := p.access()
	if p.match(EQUAL) {
		equal := p.previous()
		value := p.assignment()

		if variable, isVar := expr.(*variableExpr); isVar {
			return &assignExpr{
				name:  variable.name,
				value: value,
			}
		} else if get, isGet := expr.(*getExpr); isGet {
			return &setExpr{
				name:   get.name,
				object: get.object,
				value:  value,
			}
		}

		p.state.fatalError(errUndefinedStmt, equal.line, 0)
	}
	return expr
}

func (p *parser) access() expr {
	expr := p.or()
	for p.matchSameLine(LEFT_BRACE) {
		slice := &accessExpr{
			object: expr,
			brace:  p.previous(),
		}
		p.slice(slice)
		expr = slice
		p.consume(RIGHT_BRACE, errors.New("Expected ']' at the end of slice"))
	}
	return expr
}

func (p *parser) slice(slice *accessExpr) {
	if p.match(COLON) {
		slice.firstColon = p.previous()
		if p.match(COLON) {
			slice.secondColon = p.previous()
			slice.third = p.expression()
		} else {
			slice.second = p.expression()
			if p.match(COLON) {
				slice.secondColon = p.previous()
				slice.third = p.expression()
			}
		}
	} else {
		slice.first = p.expression()
		if p.match(COLON) {
			slice.firstColon = p.previous()
			if p.match(COLON) {
				slice.secondColon = p.previous()
				slice.third = p.expression()
			} else if !p.check(RIGHT_BRACE) && !p.isAtEnd() {
				slice.second = p.expression()
				if p.match(COLON) {
					slice.secondColon = p.previous()
					if !p.check(RIGHT_BRACE) && !p.isAtEnd() {
						slice.third = p.expression()
					}
				}
			}
		}
	}
}

func (p *parser) or() expr {
	expr := p.and()
	for p.match(OR) {
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
	for p.match(AND) {
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
	for p.match(EQUAL_EQUAL, BANG_EQUAL) {
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
	for p.match(GREATER, GREATER_EQUAL, LESS, LESS_EQUAL) {
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
	for p.match(PLUS, MINUS) {
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
	for p.match(SLASH, STAR) {
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
	for p.match(POWER) {
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
	if p.match(NOT, MINUS) {
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
		if p.match(LEFT_PAREN) {
			expr = p.finishCall(expr)
		} else if p.match(DOT) {
			name := p.consume(IDENTIFIER, errExpectedProp)
			expr = &getExpr{
				object: expr,
				name:   name,
			}
		} else {
			break
		}
	}
	return expr
}

func (p *parser) finishCall(callee expr) expr {
	arguments := p.arguments(RIGHT_PAREN)
	paren := p.consume(RIGHT_PAREN, errors.New("Expect ')' after arguments"))
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
			if tk == RIGHT_PAREN && len(arguments) >= maxFunctionParams {
				p.state.fatalError(errMaxArguments, p.peek().line, 0)
			}
			arguments = append(arguments, p.expression())
			if !p.match(COMMA) {
				break
			}
		}
	}
	return arguments
}

func (p *parser) primary() expr {
	if p.match(NUMBER, STRING) {
		return &literalExpr{value: p.previous().literal}
	}
	if p.match(FALSE) {
		return &literalExpr{value: false}
	}
	if p.match(TRUE) {
		return &literalExpr{value: true}
	}
	if p.match(NIL) {
		return &literalExpr{value: nil}
	}
	if p.match(IDENTIFIER) {
		return &variableExpr{name: p.previous()}
	}
	if p.match(LEFT_PAREN) {
		expr := p.expression()
		p.consume(RIGHT_PAREN, errUnclosedParen)
		return &groupingExpr{expression: expr}
	}
	if p.match(LEFT_BRACE) {
		return p.list()
	}
	if p.match(LEFT_CURLY_BRACE) {
		return p.dictionary()
	}

	p.state.fatalError(errUndefinedExpr, p.peek().line, 0)
	return &literalExpr{}
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

func (p *parser) matchSameLine(tokens ...tokenType) bool {
	for _, token := range tokens {
		if !p.isAtEnd() && p.peek().token == token {
			p.current++
			return true
		}
	}
	return false
}

func (p *parser) check(token tokenType) bool {
	if p.isAtEnd() {
		return false
	}
	oldCurrent := p.current
	for token != NEWLINE && !p.isAtEnd() && p.peek().token == NEWLINE {
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
	return &p.state.tokens[p.current-1]
}

func (p *parser) isAtEnd() bool {
	return p.peek().token == EOF
}

func (p *parser) synchronize() {
	p.advance()
	for !p.isAtEnd() {
		switch p.peek().token {
		case BEGIN:
			return
		case CLASS:
			return
		case FN:
			return
		case LET:
			return
		case FOR:
			return
		case IF:
			return
		case WHILE:
			return
		case RETURN:
			return
		default:
		}

		p.advance()
	}
}
