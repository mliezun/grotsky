package internal

type env struct {
	enclosing *env
	values    map[string]interface{}
}

func newEnv(enclosing *env) *env {
	return &env{
		enclosing: enclosing,
		values:    make(map[string]interface{}),
	}
}

func (e *env) get(state *interpreterState[R], name *token) interface{} {
	if value, ok := e.values[name.lexeme]; ok {
		return value
	}
	if e.enclosing != nil {
		return e.enclosing.get(state, name)
	}
	state.runtimeErr(errUndefinedVar, name)
	return nil
}

func (e *env) define(name string, value interface{}) {
	e.values[name] = value
}

func (e *env) assign(state *interpreterState[R], name *token, value interface{}) {
	if _, ok := e.values[name.lexeme]; ok {
		e.values[name.lexeme] = value
		return
	}
	if e.enclosing != nil {
		e.enclosing.assign(state, name, value)
		return
	}
	state.runtimeErr(errUndefinedVar, name)
}
