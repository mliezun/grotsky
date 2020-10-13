package internal

import "fmt"

// Printer ...
type Printer interface {
	Println(a ...interface{}) (n int, err error)
}

type stdPrinter struct{}

func (s stdPrinter) Println(a ...interface{}) (n int, err error) {
	return fmt.Println(a...)
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
