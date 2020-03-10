package internal

import (
	"strconv"
)

type Lexer struct {
	state *InterpreterState

	start   int
	current int
	line    int

	tokens []Token
}

var keywords = map[string]TokenType{
	"and":    AND,
	"class":  CLASS,
	"else":   ELSE,
	"false":  FALSE,
	"fn":     FN,
	"for":    FOR,
	"if":     IF,
	"elif":   ELIF,
	"nil":    NIL,
	"or":     OR,
	"return": RETURN,
	"super":  SUPER,
	"this":   THIS,
	"true":   TRUE,
	"let":    LET,
	"while":  WHILE,
	"not":    NOT,
	"in":     IN,
}

func NewLexer(state *InterpreterState) *Lexer {
	return &Lexer{
		state: state,
		line:  1,
	}
}

func (l *Lexer) Scan() []Token {
	for !l.isAtEnd() {
		l.start = l.current
		l.scanToken()
	}
	return l.tokens
}

func (l *Lexer) scanToken() {
	c := l.advance()
	switch c {
	case '[':
		l.emit(LEFT_BRACE, nil)
	case ']':
		l.emit(RIGHT_BRACE, nil)
	case '{':
		l.emit(LEFT_CURLY_BRACE, nil)
	case '}':
		l.emit(RIGHT_CURLY_BRACE, nil)
	case '(':
		l.emit(LEFT_PAREN, nil)
	case ')':
		l.emit(RIGHT_PAREN, nil)
	case ',':
		l.emit(COMMA, nil)
	case '.':
		l.emit(DOT, nil)
	case '-':
		l.emit(MINUS, nil)
	case '+':
		l.emit(PLUS, nil)
	case '/':
		l.emit(SLASH, nil)
	case '*':
		l.emit(STAR, nil)
	case '^':
		l.emit(POWER, nil)
	case ':':
		l.emit(COLON, nil)
	case '#':
		for !l.match('\n') && !l.isAtEnd() {
			l.advance()
		}
	case '!':
		if l.match('=') {
			l.emit(BANG_EQUAL, nil)
		} else {
			l.state.setError(WrongBang, l.line, l.start)
		}
	case '=':
		if l.match('=') {
			l.emit(EQUAL_EQUAL, nil)
		} else {
			l.emit(EQUAL, nil)
		}
	case '<':
		if l.match('=') {
			l.emit(LESS_EQUAL, nil)
		} else {
			l.emit(LESS, nil)
		}
	case '>':
		if l.match('=') {
			l.emit(GREATER_EQUAL, nil)
		} else {
			l.emit(GREATER, nil)
		}

	// Ignore whitespace
	case ' ':
	case '\r':
	case '\t':

	case '\n':
		l.line++

	case '"':
		l.string()

	default:
		if l.isDigit(c) {
			l.number()
		} else if l.isAlpha(c) {
			l.identifier()
		} else {
			l.state.setError(IllegalChar, l.line, l.start)
		}
	}
}

func (l *Lexer) string() {
	for !l.isAtEnd() && !l.match('"') {
		if l.match('\n') {
			l.line++
		}
		l.advance()
	}

	if l.isAtEnd() {
		l.state.setError(UnclosedString, l.line, l.start)
	}

	literal := l.state.Source[l.start+1 : l.current]

	// Consume ending "
	l.advance()

	l.emit(STRING, literal)
}

func (l *Lexer) number() {
	for l.isDigit(l.next()) {
		l.advance()
	}

	if l.match('.') {
		l.advance()
		for l.isDigit(l.next()) {
			l.advance()
		}
	}

	literal, _ := strconv.ParseFloat(l.state.Source[l.start:l.current], 64)

	l.emit(NUMBER, literal)
}

func (l *Lexer) identifier() {
	for l.isAlpha(l.next()) {
		l.advance()
	}

	identifier := l.state.Source[l.start:l.current]

	tokenType, ok := keywords[identifier]
	if !ok {
		tokenType = IDENTIFIER
	}

	l.emit(tokenType, nil)
}

func (l *Lexer) advance() rune {
	current := l.state.Source[l.current]
	l.current++
	return rune(current)
}

func (l *Lexer) match(c rune) bool {
	current := l.state.Source[l.current]
	return rune(current) == c
}

func (l *Lexer) emit(token TokenType, literal interface{}) {
	l.tokens = append(l.tokens, Token{
		token:   token,
		lexeme:  l.state.Source[l.start:l.current],
		literal: literal,
		line:    l.line,
	})
}

func (l *Lexer) isAtEnd() bool {
	return l.current >= len(l.state.Source)
}

func (l *Lexer) next() rune {
	return rune(l.state.Source[l.current])
}

func (l *Lexer) isDigit(c rune) bool {
	return c >= '0' && c <= '9'
}

func (l *Lexer) isAlpha(c rune) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
