package internal

import (
	"fmt"
	"io"
)

// Printer ...
type Printer interface {
	Println(a ...interface{}) (n int, err error)
	Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error)
	Fprintln(w io.Writer, a ...interface{}) (n int, err error)
}

type stdPrinter struct{}

func (s stdPrinter) Println(a ...interface{}) (n int, err error) {
	return fmt.Println(a...)
}

func (s stdPrinter) Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error) {
	return fmt.Fprintf(w, format, a...)
}

func (s stdPrinter) Fprintln(w io.Writer, a ...interface{}) (n int, err error) {
	return fmt.Fprintln(w, a...)
}

// RunSource runs source code on a fresh interpreter instance
func RunSource(source string) {
	RunSourceWithPrinter(source, stdPrinter{})
}

// RunSourceWithPrinter runs source code on a fresh interpreter instance
func RunSourceWithPrinter(source string, p Printer) {
	previousState := state
	defer func() {
		state = previousState
	}()

	state = interpreterState{source: source, errors: make([]parseError, 0)}
	lexer := &lexer{
		line: 1,
	}
	parser := &parser{}

	// TODO: ensure fresh start for each invocation, right now in test
	// global variables are still declared when a new test is run

	defineGlobals(exec.globals, p)

	lexer.scan()

	if state.PrintErrors() {
		return
	}

	parser.parse()

	if state.PrintErrors() {
		return
	}

	exec.interpret()
}
