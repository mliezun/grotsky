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
	msgs  []string
}

type returnValue struct {
	value interface{}
}

// state stores the state of a interpreter
type interpreterState struct {
	errors       []parseError
	source       string
	tokens       []token
	stmts        []stmt
	runtimeError *runtimeError
	logger       Printer
}

var state = interpreterState{}

func (interpreterState) setError(err error, line, pos int) {
	state.errors = append(state.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
}

func (interpreterState) fatalError(err error, line, pos int) {
	state.errors = append(state.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
	panic(err)
}

func (interpreterState) runtimeErr(err error, token *token, msgs ...string) {
	state.runtimeError = &runtimeError{
		err:   err,
		token: token,
		msgs:  msgs,
	}
	state.logger.Fprintf(
		os.Stderr,
		"Runtime Error on line %d\n\t%s: %s\n",
		state.runtimeError.token.line,
		state.runtimeError.err.Error(),
		state.runtimeError.token.lexeme,
	)
	panic(err)
}

// Valid returns true if the interpreter is in a valid states else false
func (interpreterState) Valid() bool {
	return len(state.errors) == 0
}

// PrintErrors prints all errors, returns true if any error printed
func (interpreterState) PrintErrors() bool {
	for _, e := range state.errors {
		state.logger.Fprintf(os.Stderr, "Error on line %d\n\t", e.line)
		state.logger.Fprintln(os.Stderr, e.err)
	}
	return len(state.errors) != 0
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
var errExpectedSemicolon = errors.New("Expected semicolon")
var errExpectedIn = errors.New("Expected 'in'")
var errExpectedFunctionName = errors.New("Expected function name")
var errExpectedParen = errors.New("Expect '(' after function name")
var errExpectedFunctionParam = errors.New("Expect function parameter")
var errMaxParameters = fmt.Errorf("Max number of parameters is %d", maxFunctionParams)
var errExpectedBegin = errors.New("Expected 'begin' at this position")
var errExpectedEnd = errors.New("Expected 'end' at this position")

// Runtime errors
var errUndefinedVar = errors.New("Undefined variable")
var errOnlyNumbers = errors.New("The operation is only defined for numbers")
var errUndefinedOp = errors.New("Undefined operation")
var errExpectedStep = errors.New("Expected step of the slice")
var errExpectedKey = errors.New("Expected key for accessing dictionary")
var errInvalidAccess = errors.New("The object is not subscriptable")
var errOnlyFunction = errors.New("Can only call functions")
var errInvalidNumberArguments = errors.New("Invalid number of arguments")
var errExpectedCollection = errors.New("Collection expected")
var errExpectedObject = errors.New("Object expected")
var errExpectedIdentifiersDict = errors.New("Expected 1 or 2 identifiers for dict")
var errCannotUnpack = errors.New("Cannot unpack value")
var errWrongNumberOfValues = errors.New("Wrong number of values to unpack")
var errMethodNotFound = errors.New("Method not found")
var errUndefinedProp = errors.New("Undefined property")
var errReadOnly = errors.New("Trying to set a property on a Read-Only object")
var errUndefinedOperator = errors.New("Undefined operator for this object")
var errExpectedNumber = errors.New("A number was expected at this position")
var errExpectedClass = errors.New("A class was expected at this position")
var errExpectedString = errors.New("A string was expected at this position")
var errExpectedFunction = errors.New("A function was expected at this position")
var errExpectedSuperclass = errors.New("Keyword 'super' is only valid inside an object")
var errExpectedDot = errors.New("Keyword 'super' is only valid for property accessing")
var errExpectedDict = errors.New("A dictionary was expected at this position")
var errExpectedList = errors.New("A list was expected at this position")
