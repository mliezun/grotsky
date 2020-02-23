package tokens

// TokenType Holds a token
type TokenType int

const (
	EOF TokenType = iota - 1

	// Single-character tokens.
	// (, ), [, ], {, } ',', ., -, +, ;, /, *, ^
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
	// return, super, this, true, let, while, not
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
)
