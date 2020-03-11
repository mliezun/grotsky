package internal

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

func (p Parser) Parse() {
	for !p.isAtEnd() {
		p.state.Stmts = append(p.state.Stmts, p.declaration())
	}
}

func (p Parser) declaration() Stmt {
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

func (p Parser) statement() Stmt {
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

func (p Parser) expressionStmt() Stmt {
	expr := p.expression()
	return StmtExpr{expression: expr}
}

func (p Parser) expression() Expr {
	if p.match(LEFT_BRACE) {
		return nil
	}
	if p.match(LEFT_CURLY_BRACE) {
		return nil
	}
	return p.assignment()
}

func (p Parser) assignment() Expr {

}

func (p Parser) or() Expr {
	expr := p.and()
	for p.match(OR) {

	}
}

func (p Parser) and() Expr {

}

func (p Parser) equality() Expr {

}

func (p Parser) comparison() Expr {

}

func (p Parser) addition() Expr {

}

func (p Parser) multiplication() Expr {

}

func (p Parser) power() Expr {

}

func (p Parser) unary() Expr {

}

func (p Parser) match(tokens ...TokenType) bool {
	for _, token := range tokens {
		if p.check(token) {
			return true
		}
	}
	return false
}

func (p Parser) check(token TokenType) bool {
	if p.isAtEnd() {
		return false
	}
	return p.peek().token == token
}

func (p Parser) peek() Token {
	return p.state.Tokens[p.current]
}

func (p Parser) previous() Token {
	return p.state.Tokens[p.current-1]
}

func (p Parser) isAtEnd() bool {
	return p.peek().token == EOF
}
