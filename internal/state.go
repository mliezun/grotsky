package internal

import (
	"errors"
	"fmt"
	"os"
	"sync"
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

var state *interpreterState
var once sync.Once

func NewInterpreter(source string) *interpreterState {
	once.Do(func() {
		state = &interpreterState{source: source, errors: make([]parseError, 0)}
	})
	return state
}

func (s *interpreterState) setError(err error, line, pos int) {
	s.errors = append(s.errors, parseError{
		err:  err,
		line: line,
		pos:  pos,
	})
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

// Posible errors
var IllegalChar = errors.New("Illegal character")
var WrongBang = errors.New("'!' cannot be used here")
var UnclosedString = errors.New("Closing \" was expected")
