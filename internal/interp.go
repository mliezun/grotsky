package internal

import (
	"fmt"
)

// RunSource runs source code on a fresh interpreter instance
func RunSource(source string) {
	previousState := state
	defer func() {
		state = previousState
	}()

	state = interpreterState{source: source, errors: make([]parseError, 0)}
	lexer := &lexer{
		line: 1,
	}
	parser := &parser{}

	var println nativeFn
	println.callFn = func(arguments []interface{}) (interface{}, error) {
		fmt.Println(arguments...)
		return nil, nil
	}
	exec.globals.define("println", &println)

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
