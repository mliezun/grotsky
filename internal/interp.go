package internal

import "fmt"

// RunSource runs source code on a fresh interpreter instance
func RunSource(source string) {
	state := &state{source: source, errors: make([]parseError, 0)}
	lexer := &lexer{
		state: state,
		line:  1,
	}
	parser := &parser{
		state: state,
	}
	env := newEnv(state, nil)
	execute := &exec{
		state:   state,
		env:     env,
		globals: env,
	}

	var println nativeFn
	println.arityFn = func() int {
		return 1
	}
	println.callFn = func(exec *exec, arguments []interface{}) interface{} {
		fmt.Println(arguments[0])
		return nil
	}
	env.define("println", &println)

	lexer.scan()

	parser.parse()

	// state.PrintTree()
	execute.interpret()
}
