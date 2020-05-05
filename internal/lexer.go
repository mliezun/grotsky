package internal

import (
	"strconv"
)

type lexer struct {
	start   int
	current int
	line    int
}

var keywords = map[string]tokenType{
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
	"begin":  BEGIN,
	"end":    END,
}

func (l *lexer) scan() {
	for !l.isAtEnd() {
		l.start = l.current
		l.scanToken()
	}
	countTokens := len(state.tokens)
	if countTokens > 0 && state.tokens[countTokens-1].token != NEWLINE {
		// Add newline if not present to terminate last statement
		state.tokens = append(state.tokens, token{
			token:   NEWLINE,
			lexeme:  "",
			literal: nil,
			line:    l.line,
		})
	}
	state.tokens = append(state.tokens, token{
		token:   EOF,
		lexeme:  "",
		literal: nil,
		line:    l.line,
	})
}

func (l *lexer) scanToken() {
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
	case ';':
		l.emit(SEMICOLON, nil)
	case '#':
		for !l.match('\n') && !l.isAtEnd() {
			l.advance()
		}
	case '!':
		if l.match('=') {
			l.advance()
			l.emit(BANG_EQUAL, nil)
		} else {
			state.setError(errWrongBang, l.line, l.start)
		}
	case '=':
		if l.match('=') {
			l.advance()
			l.emit(EQUAL_EQUAL, nil)
		} else {
			l.emit(EQUAL, nil)
		}
	case '<':
		if l.match('=') {
			l.advance()
			l.emit(LESS_EQUAL, nil)
		} else {
			l.emit(LESS, nil)
		}
	case '>':
		if l.match('=') {
			l.advance()
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
		l.emit(NEWLINE, nil)

	case '"':
		l.string()

	default:
		if l.isDigit(c) {
			l.number()
		} else if l.isAlpha(c) {
			l.identifier()
		} else {
			state.setError(errIllegalChar, l.line, l.start)
		}
	}
}

func (l *lexer) string() {
	for !l.isAtEnd() && !l.match('"') {
		if l.match('\n') {
			l.line++
		}
		l.advance()
	}

	if l.isAtEnd() {
		state.setError(errUnclosedString, l.line, l.start)
	}

	literal := state.source[l.start+1 : l.current]

	// Consume ending "
	l.advance()

	l.emit(STRING, literal)
}

func (l *lexer) number() {
	for !l.isAtEnd() && l.isDigit(l.next()) {
		l.advance()
	}

	if l.match('.') {
		l.advance()
		for l.isDigit(l.next()) {
			l.advance()
		}
	}

	literal, _ := strconv.ParseFloat(state.source[l.start:l.current], 64)

	l.emit(NUMBER, grotskyNumber(literal))
}

func (l *lexer) identifier() {
	for !l.isAtEnd() && l.isAlpha(l.next()) {
		l.advance()
	}

	identifier := state.source[l.start:l.current]

	tokenType, ok := keywords[identifier]
	if !ok {
		tokenType = IDENTIFIER
	}

	l.emit(tokenType, nil)
}

func (l *lexer) advance() rune {
	current := state.source[l.current]
	l.current++
	return rune(current)
}

func (l *lexer) match(c rune) bool {
	if l.isAtEnd() {
		return false
	}
	current := state.source[l.current]
	return rune(current) == c
}

func (l *lexer) emit(tk tokenType, literal interface{}) {
	state.tokens = append(state.tokens, token{
		token:   tk,
		lexeme:  state.source[l.start:l.current],
		literal: literal,
		line:    l.line,
	})
}

func (l *lexer) isAtEnd() bool {
	return l.current >= len(state.source)
}

func (l *lexer) next() rune {
	return rune(state.source[l.current])
}

func (l *lexer) isDigit(c rune) bool {
	return c >= '0' && c <= '9'
}

func (l *lexer) isAlpha(c rune) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
