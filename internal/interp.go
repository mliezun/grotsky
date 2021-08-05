package internal

import (
	"io"
	"sync"
)

// IPrinter printer interface
type IPrinter interface {
	Println(a ...interface{}) (n int, err error)
	Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error)
	Fprintln(w io.Writer, a ...interface{}) (n int, err error)
}

// RunSourceWithPrinter runs source code on a fresh interpreter instance
func RunSourceWithPrinter(absPath, source string, p IPrinter) bool {
	state := interpreterState{
		absPath: absPath,
		source:  source,
		errors:  make([]parseError, 0),
		logger:  p,
	}
	lexer := &lexer{
		line:  1,
		state: &state,
	}
	parser := &parser{
		state: &state,
	}

	exec = execute{
		env:   newEnv(nil),
		mx:    &sync.Mutex{},
		state: &state,
	}
	exec.globals = exec.env

	defineGlobals(&state, exec.globals, p)

	lexer.scan()

	if state.PrintErrors() {
		return false
	}

	parser.parse()

	if state.PrintErrors() {
		return false
	}

	// exec.mx.Lock()

	defer state.PrintErrors()

	return exec.interpret()
}

func importModule(previousState *interpreterState, absPath, moduleSource string) (*env, bool) {
	p := previousState.logger

	state := interpreterState{
		absPath: absPath,
		source:  moduleSource,
		errors:  make([]parseError, 0),
		logger:  p,
	}
	lexer := &lexer{
		line:  1,
		state: &state,
	}
	parser := &parser{
		state: &state,
	}

	oldExec := exec
	defer func() {
		exec = oldExec
	}()

	moduleEnv := newEnv(nil)
	exec = execute{
		env:   moduleEnv,
		mx:    &sync.Mutex{},
		state: &state,
	}
	exec.globals = exec.env

	defineGlobals(&state, exec.globals, p)

	lexer.scan()

	if state.PrintErrors() {
		return nil, false
	}

	parser.parse()

	if state.PrintErrors() {
		return nil, false
	}

	// exec.mx.Lock()

	defer state.PrintErrors()

	if !exec.interpret() {
		return nil, false
	}

	return moduleEnv, true
}
