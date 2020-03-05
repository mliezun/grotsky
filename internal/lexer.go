package internal

import (
	"strconv"
)

type Lexer struct {
	source  string
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

func NewLexer(source string) *Lexer {
	return &Lexer{
		source: source,
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
			// TODO: handle error
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
		if l.isDigit() {
			l.number()
		} else if l.isAlpha() {
			l.identifier()
		} else {
			// TODO: handle error
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
		// TODO: handle error
	}

	literal := l.source[l.start+1 : l.current]

	// Consume ending "
	l.advance()

	l.emit(STRING, literal)
}

func (l *Lexer) number() {
	for l.isDigit() {
		l.advance()
	}

	if l.match('.') {
		l.advance()
		for l.isDigit() {
			l.advance()
		}
	}

	literal, _ := strconv.ParseFloat(l.source[l.start:l.current], 64)

	l.emit(NUMBER, literal)
}

func (l *Lexer) identifier() {
	for l.isAlpha() {
		l.advance()
	}

	identifier := l.source[l.start:l.current]

	tokenType, ok := keywords[identifier]
	if !ok {
		tokenType = IDENTIFIER
	}

	l.emit(tokenType, nil)
}

func (l *Lexer) advance() rune {
	current := l.source[l.current]
	l.current++
	return rune(current)
}

func (l *Lexer) match(c rune) bool {
	current := l.source[l.current]
	return rune(current) == c
}

func (l *Lexer) emit(token TokenType, literal interface{}) {
	l.tokens = append(l.tokens, Token{
		token:   token,
		lexeme:  l.source[l.start:l.current],
		literal: literal,
		line:    l.line,
	})
}

func (l *Lexer) isAtEnd() bool {
	return l.current >= len(l.source)
}

func (l *Lexer) isDigit() bool {
	c := rune(l.source[l.current])
	return c >= '0' && c <= '9'
}

func (l *Lexer) isAlpha() bool {
	c := rune(l.source[l.current])
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
