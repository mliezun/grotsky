package internal

import (
	"errors"
	"fmt"
	"os"
	"sync"
)

// ParseError stores a parser error
type ParseError struct {
	Error error
	Line  int
	Pos   int
}

// InterpreterState stores the error state of a interpreter
type InterpreterState struct {
	Errors []ParseError
	Source string
}

var state *InterpreterState
var once sync.Once

func NewInterpreterState(source string) *InterpreterState {
	once.Do(func() {
		state = &InterpreterState{Source: source, Errors: make([]ParseError, 0)}
	})
	return state
}

func (s *InterpreterState) setError(err error, line, pos int) {
	s.Errors = append(s.Errors, ParseError{
		Error: err,
		Line:  line,
		Pos:   pos,
	})
}

// Valid returns true if the interpreter is in a valid states else false
func (s *InterpreterState) Valid() bool {
	return len(s.Errors) == 0
}

// PrintErrors prints all errors
func (s *InterpreterState) PrintErrors() {
	for _, e := range s.Errors {
		fmt.Fprintf(os.Stderr, "Error on line %d\n", e.Line)
		fmt.Fprintln(os.Stderr, e.Error)
	}
}

// Posible errors
var IllegalChar = errors.New("Illegal character")
var WrongBang = errors.New("'!' cannot be used here")
var UnclosedString = errors.New("Closing \" was expected")
