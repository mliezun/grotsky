package internal

type tokenType int

const (
	EOF tokenType = iota - 1
	NEWLINE

	// Single-character tokens.
	// (, ), [, ], {, } ',', ., -, +, ;, /, *, ^, :
	LEFT_PAREN
	RIGHT_PAREN
	LEFT_BRACE
	RIGHT_BRACE
	RIGHT_CURLY_BRACE
	LEFT_CURLY_BRACE
	COMMA
	DOT
	MINUS
	PLUS
	SLASH
	STAR
	POWER
	COLON

	// One or two character tokens.
	// !=, =, ==, >, >=, <, <=
	BANG_EQUAL
	EQUAL
	EQUAL_EQUAL
	GREATER
	GREATER_EQUAL
	LESS
	LESS_EQUAL

	// Literals.
	// *variable*, string, int
	IDENTIFIER
	STRING
	NUMBER

	// Keywords.
	// and, class, else, false, fn, for, if, elif, nil, or,
	// return, super, this, true, let, while, not, in, begin, end
	AND
	CLASS
	ELSE
	FALSE
	FN
	FOR
	IF
	ELIF
	NIL
	OR
	RETURN
	SUPER
	THIS
	TRUE
	LET
	WHILE
	NOT
	IN
	BEGIN
	END
)

type token struct {
	token   tokenType
	lexeme  string
	literal interface{}
	line    int
}
