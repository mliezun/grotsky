package internal

import (
	"strconv"
)

type lexer struct {
	start   int
	current int
	line    int

	state *interpreterState
}

var keywords = map[string]tokenType{
	"and":      tkAnd,
	"class":    tkClass,
	"else":     tkElse,
	"false":    tkFalse,
	"fn":       tkFn,
	"for":      tkFor,
	"if":       tkIf,
	"elif":     tkElif,
	"nil":      tkNil,
	"or":       tkOr,
	"return":   tkReturn,
	"break":    tkBreak,
	"continue": tkContinue,
	"super":    tkSuper,
	"this":     tkThis,
	"true":     tkTrue,
	"let":      tkLet,
	"while":    tkWhile,
	"not":      tkNot,
	"in":       tkIn,
	"try":      tkTry,
	"catch":    tkCatch,
}

func (l *lexer) scan() {
	for !l.isAtEnd() {
		l.start = l.current
		l.scanToken()
	}
	countTokens := len(l.state.tokens)
	if countTokens > 0 && l.state.tokens[countTokens-1].token != tkNewline {
		// Add newline if not present to terminate last statement
		l.state.tokens = append(l.state.tokens, token{
			token:   tkNewline,
			lexeme:  "",
			literal: nil,
			line:    l.line,
		})
	}
	l.state.tokens = append(l.state.tokens, token{
		token:   tkEOF,
		lexeme:  "",
		literal: nil,
		line:    l.line,
	})
}

func (l *lexer) scanToken() {
	c := l.advance()
	switch c {
	case '[':
		l.emit(tkLeftBrace, nil)
	case ']':
		l.emit(tkRightBrace, nil)
	case '{':
		l.emit(tkLeftCurlyBrace, nil)
	case '}':
		l.emit(tkRightCurlyBrace, nil)
	case '(':
		l.emit(tkLeftParen, nil)
	case ')':
		l.emit(tkRightParen, nil)
	case ',':
		l.emit(tkComma, nil)
	case '.':
		l.emit(tkDot, nil)
	case '-':
		l.emit(tkMinus, nil)
	case '+':
		l.emit(tkPlus, nil)
	case '/':
		l.emit(tkSlash, nil)
	case '%':
		l.emit(tkMod, nil)
	case '*':
		l.emit(tkStar, nil)
	case '^':
		l.emit(tkPower, nil)
	case ':':
		l.emit(tkColon, nil)
	case ';':
		l.emit(tkSemicolon, nil)
	case '#':
		for !l.match('\n') && !l.isAtEnd() {
			l.advance()
		}
	case '!':
		if l.match('=') {
			l.advance()
			l.emit(tkBangEqual, nil)
		} else {
			l.state.setError(errWrongBang, l.line, l.start)
		}
	case '=':
		if l.match('=') {
			l.advance()
			l.emit(tkEqualEqual, nil)
		} else {
			l.emit(tkEqual, nil)
		}
	case '<':
		if l.match('=') {
			l.advance()
			l.emit(tkLessEqual, nil)
		} else {
			l.emit(tkLess, nil)
		}
	case '>':
		if l.match('=') {
			l.advance()
			l.emit(tkGreaterEqual, nil)
		} else {
			l.emit(tkGreater, nil)
		}

	// Ignore whitespace
	case ' ':
	case '\r':
	case '\t':

	case '\n':
		l.line++
		l.emit(tkNewline, nil)

	case '"':
		l.string()

	default:
		if l.isDigit(c) {
			l.number()
		} else if l.isAlpha(c) {
			l.identifier()
		} else {
			l.state.setError(errIllegalChar, l.line, l.start)
		}
	}
}

func (l *lexer) unescapeSequence() grotskyString {
	if l.isAtEnd() {
		return grotskyString("")
	}
	c := l.advance()
	switch c {
	case 'a':
		return grotskyString("\a")
	case 'b':
		return grotskyString("\b")
	case 'f':
		return grotskyString("\f")
	case 'n':
		return grotskyString("\n")
	case 'r':
		return grotskyString("\r")
	case 't':
		return grotskyString("\t")
	case 'v':
		return grotskyString("\v")
	case '\\':
		return grotskyString("\\")
	case '"':
		return grotskyString("\"")
	default:
		return grotskyString("")
	}
}

func (l *lexer) string() {
	literal := grotskyString("")
	for !l.isAtEnd() && !l.match('"') {
		if l.match('\n') {
			l.line++
		}
		if l.match('\\') {
			l.advance()
			unescaped := l.unescapeSequence()
			if unescaped == "" {
				l.state.setError(errUnclosedString, l.line, l.start)
				return
			}
			literal += unescaped
			continue
		}
		literal += grotskyString(l.state.source[l.current])
		l.advance()
	}

	if l.isAtEnd() {
		l.state.setError(errUnclosedString, l.line, l.start)
		return
	}

	// Consume ending "
	l.advance()

	l.emit(tkString, literal)
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

	literal, _ := strconv.ParseFloat(l.state.source[l.start:l.current], 64)

	l.emit(tkNumber, grotskyNumber(literal))
}

func (l *lexer) identifier() {
	for !l.isAtEnd() && l.isAlpha(l.next()) {
		l.advance()
	}

	identifier := l.state.source[l.start:l.current]

	tokenType, ok := keywords[identifier]
	if !ok {
		tokenType = tkIdentifier
	}

	l.emit(tokenType, nil)
}

func (l *lexer) advance() rune {
	current := l.state.source[l.current]
	l.current++
	return rune(current)
}

func (l *lexer) match(c rune) bool {
	if l.isAtEnd() {
		return false
	}
	current := l.state.source[l.current]
	return rune(current) == c
}

func (l *lexer) emit(tk tokenType, literal interface{}) {
	l.state.tokens = append(l.state.tokens, token{
		token:   tk,
		lexeme:  l.state.source[l.start:l.current],
		literal: literal,
		line:    l.line,
	})
}

func (l *lexer) isAtEnd() bool {
	return l.current >= len(l.state.source)
}

func (l *lexer) next() rune {
	return rune(l.state.source[l.current])
}

func (l *lexer) isDigit(c rune) bool {
	return c >= '0' && c <= '9'
}

func (l *lexer) isAlpha(c rune) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
