package internal

import (
	"errors"
)

// Parser stores parser data
type Parser struct {
	state   *InterpreterState
	current int
}

func NewParser(state *InterpreterState) *Parser {
	return &Parser{
		state: state,
	}
}

func (p *Parser) Parse() {
	for !p.isAtEnd() {
		p.state.Stmts = append(p.state.Stmts, p.declaration())
	}
}

func (p *Parser) declaration() Stmt {
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

func (p *Parser) statement() Stmt {
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

func (p *Parser) expressionStmt() Stmt {
	expr := p.expression()
	return &ExprStmt{expression: expr}
}

func (p *Parser) expression() Expr {
	if p.match(LEFT_BRACE) {
		return nil
	}
	if p.match(LEFT_CURLY_BRACE) {
		return nil
	}
	return p.assignment()
}

func (p *Parser) assignment() Expr {
	expr := p.or()
	if p.match(EQUAL) {

	}
	return expr
}

func (p *Parser) or() Expr {
	expr := p.and()
	for p.match(OR) {

	}
	return expr
}

func (p *Parser) and() Expr {
	expr := p.equality()
	for p.match(AND) {

	}
	return expr
}

func (p *Parser) equality() Expr {
	expr := p.comparison()
	for p.match(EQUAL_EQUAL, BANG_EQUAL) {

	}
	return expr
}

func (p *Parser) comparison() Expr {
	expr := p.addition()
	for p.match(GREATER, GREATER_EQUAL, LESS, LESS_EQUAL) {

	}
	return expr
}

func (p *Parser) addition() Expr {
	expr := p.multiplication()
	for p.match(PLUS, MINUS) {
		operator := p.previous()
		right := p.multiplication()
		expr = &BinaryExpr{
			left:     expr,
			operator: operator,
			right:    right,
		}
	}
	return expr
}

func (p *Parser) multiplication() Expr {
	expr := p.power()
	for p.match(SLASH, STAR) {

	}
	return expr
}

func (p *Parser) power() Expr {
	expr := p.unary()
	for p.match(POWER) {

	}
	return expr
}

func (p *Parser) unary() Expr {
	if p.match(NOT) {
		operator := p.previous()
		right := p.unary()
		return &UnaryExpr{
			operator: operator,
			right:    right,
		}
	}
	return p.call()
}

func (p *Parser) call() Expr {
	expr := p.access()
	for {
		if p.match(LEFT_PAREN) {
			expr = p.finishCall(expr)
		} else if p.match(DOT) {
			//TODO: set correct error
			name := p.consume(IDENTIFIER, errors.New("Expect property name after '.'"))
			expr = &GetExpr{
				object: expr,
				name:   name,
			}
		} else {
			break
		}
	}
	return expr
}

func (p *Parser) finishCall(callee Expr) Expr {
	arguments := make([]Expr, 0)
	if !p.check(RIGHT_PAREN) {
		for {
			if len(arguments) >= 255 {
				//TODO: handle error
			}
			arguments = append(arguments, p.expression())
		}
	}
	paren := p.consume(RIGHT_PAREN, errors.New("Expect ')' after arguments"))
	return &CallExpr{
		callee:    callee,
		arguments: arguments,
		paren:     paren,
	}
}

func (p *Parser) access() Expr {
	expr := p.primary()
	if p.match(LEFT_BRACE) {

	}
	return expr
}

func (p *Parser) primary() Expr {
	if p.match(NUMBER, STRING) {
		return &LiteralExpr{value: p.previous().literal}
	}
	if p.match(FALSE) {
		return &LiteralExpr{value: false}
	}
	if p.match(TRUE) {
		return &LiteralExpr{value: true}
	}
	if p.match(NIL) {
		return &LiteralExpr{value: nil}
	}
	if p.match(IDENTIFIER) {
		return &VariableExpr{name: p.previous()}
	}
	if p.match(LEFT_PAREN) {
		expr := p.expression()
		// TODO: set correct error
		p.consume(RIGHT_PAREN, errors.New("Expect ')' after expression"))
		return &GroupingExpr{expression: expr}
	}

	// TODO: handle error
	return &LiteralExpr{}
}

func (p *Parser) consume(token TokenType, err error) *Token {
	if p.check(token) {
		return p.advance()
	}

	p.state.setError(err, 0, 0)
	return &Token{}
}

func (p *Parser) advance() *Token {
	if !p.isAtEnd() {
		p.current++
	}
	return p.previous()
}

func (p *Parser) match(tokens ...TokenType) bool {
	for _, token := range tokens {
		if p.check(token) {
			p.current++
			return true
		}
	}
	return false
}

func (p *Parser) check(token TokenType) bool {
	if p.isAtEnd() {
		return false
	}
	return p.peek().token == token
}

func (p *Parser) peek() Token {
	return p.state.Tokens[p.current]
}

func (p *Parser) previous() *Token {
	return &p.state.Tokens[p.current-1]
}

func (p *Parser) isAtEnd() bool {
	return p.peek().token == EOF
}
