package internal

import "fmt"

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
	obj := &grotskyObject{
		class:  c,
		fields: make(map[string]interface{}),
	}
	if init := c.findMethod("init"); init != nil {
		if _, err := init.bind(obj).call(arguments); err != nil {
			return nil, err
		}
	}
	return obj, nil
}

func (c *grotskyClass) get(state *interpreterState[R], tk *token) interface{} {
	if method, ok := c.staticMethods[tk.lexeme]; ok {
		return method
	}

	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (c *grotskyClass) set(state *interpreterState[R], name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (c *grotskyClass) getOperator(op operator) (operatorApply, error) {
	return nil, errUndefinedOp
}

func (c *grotskyClass) String() string {
	extends := ""
	if c.superclass != nil {
		extends = " extends " + c.superclass.name
	}
	return fmt.Sprintf("<class %s%s>", c.name, extends)
}
