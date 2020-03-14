package internal

import (
	"errors"
)

// parser stores parser data
type parser struct {
	state   *interpreterState
	current int
}

func (state *interpreterState) Parse() {
	p := &parser{
		state: state,
	}
	for !p.isAtEnd() {
		p.state.stmts = append(p.state.stmts, p.declaration())
	}
}

func (p *parser) declaration() stmt {
	if p.match(CLASS) {
		return nil
	}
	if p.match(FN) {
		return nil
	}
	if p.match(FN) {
		return nil
	}
	return p.statement()
}

func (p *parser) statement() stmt {
	if p.match(FOR) {
		return nil
	}
	if p.match(IF) {
		return nil
	}
	if p.match(RETURN) {
		return nil
	}
	if p.match(WHILE) {
		return nil
	}
	return p.expressionStmt()
}

func (p *parser) expressionStmt() stmt {
	expr := p.expression()
	if !p.isAtEnd() {
		p.consume(NEWLINE, errors.New("Expected new line at the end of statement"))
	}
	return &exprStmt{expression: expr}
}

func (p *parser) expression() expr {
	return p.assignment()
}

func (p *parser) list() expr {
	elements := p.arguments(RIGHT_BRACE)
	//TODO: set correct error
	brace := p.consume(RIGHT_BRACE, errors.New("Expected ']' at end of list"))
	return &listExpr{
		elements: elements,
		brace:    brace,
	}
}

func (p *parser) dictionary() expr {
	elements := p.dictElements()
	//TODO: set correct error
	curlyBrace := p.consume(RIGHT_CURLY_BRACE, errors.New("Expected '}' at the end of dict"))
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
		//TODO: set correct error
		p.consume(COLON, errors.New("Expected ':' after key"))
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
		equals := p.previous()
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

		//TODO: handle error
		//TODO: remove this if
		if equals.lexeme != "==" {
			return expr
		}
	}
	return expr
}

func (p *parser) access() expr {
	expr := p.or()
	for p.matchSameLine(LEFT_BRACE) {
		slice := p.slice()
		expr = &accessExpr{
			object: expr,
			slice:  slice,
		}
		p.consume(RIGHT_BRACE, errors.New("Expected ']' at the end of slice"))
	}
	return expr
}

func (p *parser) slice() expr {
	slice := &sliceExpr{}
	if p.match(COLON) {
		slice.firstColon = p.previous()
		if p.match(COLON) {
			slice.secondColon = p.previous()
			slice.third = p.expression()
		} else {
			slice.first = p.expression()
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
	return slice
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
			//TODO: set correct error
			name := p.consume(IDENTIFIER, errors.New("Expect property name after '.'"))
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
			if tk == RIGHT_PAREN && len(arguments) >= 255 {
				//TODO: handle error
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
		// TODO: set correct error
		p.consume(RIGHT_PAREN, errors.New("Expect ')' after expression"))
		return &groupingExpr{expression: expr}
	}
	if p.match(LEFT_BRACE) {
		return p.list()
	}
	if p.match(LEFT_CURLY_BRACE) {
		return p.dictionary()
	}

	// TODO: handle error
	return &literalExpr{}
}

func (p *parser) consume(tk tokenType, err error) *token {
	if p.check(tk) {
		return p.advance()
	}

	p.state.setError(err, 0, 0)
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
	ignoreNewLine := token != NEWLINE
	oldCurrent := p.current
	for ignoreNewLine && !p.isAtEnd() && p.peek().token == NEWLINE {
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
