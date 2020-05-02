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

type runtimeError struct {
	err   error
	token *token
}

type returnValue interface{}

// state stores the state of a interpreter
type state struct {
	errors       []parseError
	source       string
	tokens       []token
	stmts        []stmt
	runtimeError *runtimeError
}

func (s *state) setError(err error, line, pos int) {
	s.errors = append(s.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
}

func (s *state) fatalError(err error, line, pos int) {
	s.errors = append(s.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
	panic(err)
}

func (s *state) runtimeErr(err error, token *token) {
	s.runtimeError = &runtimeError{
		err:   err,
		token: token,
	}
	panic(err)
}

// Valid returns true if the interpreter is in a valid states else false
func (s *state) Valid() bool {
	return len(s.errors) == 0
}

// PrintErrors prints all errors, returns true if any error printed
func (s *state) PrintErrors() bool {
	for _, e := range s.errors {
		fmt.Fprintf(os.Stderr, "Error on line %d\n", e.line)
		fmt.Fprintln(os.Stderr, e.err)
	}
	return len(s.errors) != 0
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
var errMaxArguments = fmt.Errorf("Max number of arguments is %d", maxFunctionParams)
var errExpectedIdentifier = errors.New("Expected variable name")
var errExpectedNewline = errors.New("Expected new line")
var errExpectedComma = errors.New("Expected comma")
var errExpectedIn = errors.New("Expected 'in'")
var errExpectedFunctionName = errors.New("Expected function name")
var errExpectedParen = errors.New("Expect '(' after function name")
var errExpectedFunctionParam = errors.New("Expect function parameter")
var errMaxParameters = fmt.Errorf("Max number of parameters is %d", maxFunctionParams)
var errExpectedBegin = errors.New("Expected 'begin' at this position")

// Runtime errors
var errUndefinedVar = errors.New("Undefined variable")
var errOnlyNumbers = errors.New("The operation is only defined for numbers")
var errUndefinedOp = errors.New("Undefined operation")
var errExpectedStep = errors.New("Expected step of the slice")
var errExpectedKey = errors.New("Expected key for accessing dictionary")
var errInvalidAccess = errors.New("The object is not subscriptable")
var errOnlyFunction = errors.New("Can only call functions")
var errInvalidNumberArguments = errors.New("Invalid number of arguments")
