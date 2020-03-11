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
	if p.match(LEFT_BRACE) {
		return nil
	}
	if p.match(LEFT_CURLY_BRACE) {
		return nil
	}
	expr := p.assignment()
	return nil
}

func (p Parser) assignment() Expr {
}

func (p Parser) match(tokens ...TokenType) bool {
	for _, token := range tokens {
		if check(token) {
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

func (p Parser) isAtEnd() bool {
	return p.peek().token == EOF
}
