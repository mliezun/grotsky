package internal

type grotskyClass struct {
	name          string
	superclass    *grotskyClass
	methods       map[string]*grotskyFunction
	staticMethods map[string]*grotskyFunction
}

func (c *grotskyClass) findMethod(name string) *grotskyFunction {
	if method, ok := c.methods[name]; ok {
		return method
	}
	if c.superclass != nil {
		return c.superclass.findMethod(name)
	}
	return nil
}

func (c *grotskyClass) arity() int {
	if init := c.findMethod("init"); init != nil {
		return init.arity()
	}
	return 0
}

func (c *grotskyClass) call(arguments []interface{}) interface{} {
	obj := &grotskyObject{class: c}
	if init := c.findMethod("init"); init != nil {
		init.bind(obj).call(arguments)
	}
	return obj
}

func (c *grotskyClass) get(tk *token) interface{} {
	if method, ok := c.staticMethods[tk.lexeme]; ok {
		return method
	}

	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (c *grotskyClass) set(name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}
