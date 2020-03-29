package internal

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
	exec := &exec{
		state:   state,
		env:     env,
		globals: env,
	}

	lexer.scan()

	parser.parse()

	// state.PrintTree()
	exec.interpret()
}
