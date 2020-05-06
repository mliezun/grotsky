package internal

import "fmt"

type nativeFn struct {
	callFn func(arguments []interface{}) (interface{}, error)
}

func (n *nativeFn) call(arguments []interface{}) (interface{}, error) {
	return n.callFn(arguments)
}

type nativeObj struct {
	getFn         func(tk *token) interface{}
	setFn         func(name *token, value interface{})
	getOperatorFn func(op operator) (operatorApply, error)
}

func (n *nativeObj) get(tk *token) interface{} {
	if n.getFn == nil {
		state.runtimeErr(errUndefinedProp, tk)
	}
	return n.getFn(tk)
}

func (n *nativeObj) set(name *token, value interface{}) {
	if n.setFn == nil {
		state.runtimeErr(errReadOnly, name)
	}
	n.setFn(name, value)
}

func (n *nativeObj) getOperator(op operator) (operatorApply, error) {
	if n.getOperatorFn == nil {
		return nil, errUndefinedOp
	}
	return n.getOperatorFn(op)
}

func defineGlobals(e *env) {
	defineIo(e)
}

func defineIo(e *env) {
	var println nativeFn
	println.callFn = func(arguments []interface{}) (interface{}, error) {
		fmt.Println(arguments...)
		return nil, nil
	}

	e.define("io", &nativeObj{
		getFn: func(tk *token) interface{} {
			switch tk.lexeme {
			case "println":
				return println
			default:
				state.runtimeErr(errUndefinedProp, tk)
			}
			return nil
		},
	})
}
