package internal

import (
	"io"
)

// IPrinter printer interface
type IPrinter interface {
	Println(a ...interface{}) (n int, err error)
	Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error)
	Fprintln(w io.Writer, a ...interface{}) (n int, err error)
}

// RunSourceWithPrinter runs source code on a fresh interpreter instance
func RunSourceWithPrinter(absPath, source string, p IPrinter) bool {
	previousState := state
	defer func() {
		state = previousState
	}()

	state = interpreterState{
		absPath: absPath,
		source:  source,
		errors:  make([]parseError, 0),
		logger:  p,
	}
	lexer := &lexer{
		line: 1,
	}
	parser := &parser{}

	exec = execute{
		env: newEnv(nil),
	}
	exec.globals = exec.env

	defineGlobals(exec.globals, p)

	lexer.scan()

	if state.PrintErrors() {
		return false
	}

	parser.parse()

	if state.PrintErrors() {
		return false
	}

	return exec.interpret()
}

func importModule(moduleSource string) (*env, bool) {
	previousState := state
	defer func() {
		state = previousState
	}()

	p := previousState.logger

	state = interpreterState{
		source: moduleSource,
		errors: make([]parseError, 0),
		logger: p,
	}
	lexer := &lexer{
		line: 1,
	}
	parser := &parser{}

	moduleEnv := newEnv(nil)
	exec = execute{
		env: moduleEnv,
	}
	exec.globals = exec.env

	defineGlobals(exec.globals, p)

	lexer.scan()

	if state.PrintErrors() {
		return nil, false
	}

	parser.parse()

	if state.PrintErrors() {
		return nil, false
	}

	if !exec.interpret() {
		return nil, false
	}

	return moduleEnv, true
}
