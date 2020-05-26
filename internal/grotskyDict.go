package internal

import "reflect"

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
		return reflect.ValueOf(x).Pointer() == reflect.ValueOf(y).Pointer()
	},
	opNeq: func(x, y map[interface{}]interface{}) interface{} {
		return reflect.ValueOf(x).Pointer() != reflect.ValueOf(y).Pointer()
	},
}

func (d grotskyDict) get(tk *token) interface{} {
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (d grotskyDict) set(name *token, value interface{}) {
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
