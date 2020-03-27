package internal

import (
	"errors"
	"fmt"
	"os"
)

type parseError struct {
	err  error
	line int
	pos  int
}

// interpreterState stores the state of a interpreter
type interpreterState struct {
	errors []parseError
	source string
	tokens []token
	stmts  []stmt
}

// NewInterpreterState creates a new interpreter state
func NewInterpreterState(source string) *interpreterState {
	state := &interpreterState{source: source, errors: make([]parseError, 0)}
	return state
}

func (s *interpreterState) setError(err error, line, pos int) {
	s.errors = append(s.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
}

func (s *interpreterState) fatalError(err error, line, pos int) {
	s.errors = append(s.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
	panic(err)
}

// Valid returns true if the interpreter is in a valid states else false
func (s *interpreterState) Valid() bool {
	return len(s.errors) == 0
}

// PrintErrors prints all errors
func (s *interpreterState) PrintErrors() {
	for _, e := range s.errors {
		fmt.Fprintf(os.Stderr, "Error on line %d\n", e.line)
		fmt.Fprintln(os.Stderr, e.err)
	}
}

// Lexer errors
var errIllegalChar = errors.New("Illegal character")
var errWrongBang = errors.New("'!' cannot be used here")
var errUnclosedString = errors.New("Closing \" was expected")

// Parser errors
var errUnclosedParen = errors.New("Expect ')' after expression")
var errUnclosedBracket = errors.New("Expected ']' at end of list")
var errUnclosedCurlyBrace = errors.New("Expected '}' at the end of dict")
var errExpectedColon = errors.New("Expected ':' after key")
var errExpectedProp = errors.New("Expected property name after '.'")
var errUndefinedExpr = errors.New("Undefined expression")
var errUndefinedStmt = errors.New("Undefined statement")
var errMaxArguments = errors.New("Max number of arguments is 255")
var errExpectedIdentifier = errors.New("Expected variable name")
var errExpectedNewline = errors.New("Expected new line")
var errExpectedComma = errors.New("Expected comma")
var errExpectedIn = errors.New("Expected 'in'")
