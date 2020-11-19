package internal

import "fmt"

type grotskyObject struct {
	class  *grotskyClass
	fields map[string]interface{}
}

type grotskyInstance interface {
	get(state *interpreterState, tk *token) interface{}
	set(state *interpreterState, name *token, value interface{})
	getOperator(op operator) (operatorApply, error)
	String() string
}

func (o *grotskyObject) get(state *interpreterState, tk *token) interface{} {
	if val, ok := o.fields[tk.lexeme]; ok {
		return val
	}
	if method := o.class.findMethod(tk.lexeme); method != nil {
		return method.bind(o)
	}
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (o *grotskyObject) set(state *interpreterState, name *token, value interface{}) {
	o.fields[name.lexeme] = value
}

func (o *grotskyObject) getOperator(op operator) (operatorApply, error) {
	if method := o.class.findMethod(string(op)); method != nil {
		boundMethod := method.bind(o)
		return func(arguments ...interface{}) (interface{}, error) {
			return boundMethod.call(arguments)
		}, nil
	}
	return nil, errUndefinedOperator
}

func (o *grotskyObject) String() string {
	return fmt.Sprintf("<instance %s>", o.class.String())
}
