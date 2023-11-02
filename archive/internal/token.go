package internal

type tokenType int

const (
	tkEOF tokenType = iota - 1
	tkNewline

	// Single-character tokens.
	// (, ), [, ], {, } ',', ., -, +, ;, /, %, *, ^, :, ;
	tkLeftParen
	tkRightParen
	tkLeftBrace
	tkRightBrace
	tkRightCurlyBrace
	tkLeftCurlyBrace
	tkComma
	tkDot
	tkMinus
	tkPlus
	tkSlash
	tkMod
	tkStar
	tkPower
	tkColon
	tkSemicolon

	// One or two character tokens.
	// !=, =, ==, >, >=, <, <=
	tkBangEqual
	tkEqual
	tkEqualEqual
	tkGreater
	tkGreaterEqual
	tkLess
	tkLessEqual

	// Literals.
	// *variable*, string, int
	tkIdentifier
	tkString
	tkNumber

	// Keywords.
	// and, class, else, false, fn, for, if, elif, nil, or,
	// return, break, continue, super, this, true, let, while, not, in, begin, end
	tkAnd
	tkClass
	tkElse
	tkFalse
	tkFn
	tkFor
	tkIf
	tkElif
	tkNil
	tkOr
	tkReturn
	tkBreak
	tkContinue
	tkSuper
	tkThis
	tkTrue
	tkLet
	tkWhile
	tkNot
	tkIn
	tkTry
	tkCatch
)

type token struct {
	token   tokenType
	lexeme  string
	literal interface{}
	line    int
}
