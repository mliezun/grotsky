package internal

type env struct {
	state *state

	enclosing *env
	values    map[string]interface{}
}

func (e *env) get(name *token) interface{} {
	if value, ok := e.values[name.lexeme]; ok {
		return value
	}
	if e.enclosing != nil {
		return e.enclosing.get(name)
	}
	e.state.runtimeErr(errUndefinedVar, name)
	return nil
}

func (e *env) define(name string, value interface{}) {
	e.values[name] = value
}

func (e *env) assign(name *token, value interface{}) {
	if _, ok := e.values[name.lexeme]; ok {
		e.values[name.lexeme] = value
		return
	}
	if e.enclosing != nil {
		e.enclosing.assign(name, value)
		return
	}
	e.state.runtimeErr(errUndefinedVar, name)
}
