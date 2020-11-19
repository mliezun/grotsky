package internal

import (
	"fmt"
	"reflect"
)

type grotskyDict map[interface{}]interface{}

func applyOpToDict(op func(x, y map[interface{}]interface{}) interface{}, arguments ...interface{}) (interface{}, error) {
	x := arguments[0].(grotskyDict)
	y, ok := arguments[1].(grotskyDict)
	if !ok {
		return nil, errExpectedDict
	}
	return op(map[interface{}]interface{}(x), map[interface{}]interface{}(y)), nil
}

var dictBinaryOperations = map[operator]func(x, y map[interface{}]interface{}) interface{}{
	opAdd: func(x, y map[interface{}]interface{}) interface{} {
		out := make(map[interface{}]interface{})
		for k, v := range x {
			out[k] = v
		}
		for k, v := range y {
			out[k] = v
		}
		return grotskyDict(out)
	},
	opEq: func(x, y map[interface{}]interface{}) interface{} {
		return grotskyBool(reflect.ValueOf(x).Pointer() == reflect.ValueOf(y).Pointer())
	},
	opNeq: func(x, y map[interface{}]interface{}) interface{} {
		return grotskyBool(reflect.ValueOf(x).Pointer() != reflect.ValueOf(y).Pointer())
	},
}

func (d grotskyDict) get(state *interpreterState, tk *token) interface{} {
	if tk.lexeme == "length" {
		return grotskyNumber(len(d))
	}
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (d grotskyDict) set(state *interpreterState, name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (d grotskyDict) getOperator(op operator) (operatorApply, error) {
	if apply, ok := dictBinaryOperations[op]; ok {
		return func(arguments ...interface{}) (interface{}, error) {
			return applyOpToDict(apply, append([]interface{}{d}, arguments...)...)
		}, nil
	}
	return nil, errUndefinedOp
}

func (d grotskyDict) String() string {
	out := "{"
	i := 0
	for key, val := range d {
		out += fmt.Sprintf("%s: %s", printObj(key), printObj(val))
		if len(d) > 1 && i != len(d)-1 {
			out += ", "
		}
		i++
	}
	return out + "}"
}
