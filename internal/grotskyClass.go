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

func (c *grotskyClass) call(arguments []interface{}) (interface{}, error) {
	obj := &grotskyObject{class: c}
	if init := c.findMethod("init"); init != nil {
		if _, err := init.bind(obj).call(arguments); err != nil {
			return nil, err
		}
	}
	return obj, nil
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
