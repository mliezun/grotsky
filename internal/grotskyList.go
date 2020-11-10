package internal

import (
	"fmt"
	"reflect"
)

type grotskyList []interface{}

func applyOpToList(op func(x, y []interface{}) interface{}, arguments ...interface{}) (interface{}, error) {
	x := arguments[0].(grotskyList)
	y, ok := arguments[1].(grotskyList)
	if !ok {
		return nil, errExpectedList
	}
	return op([]interface{}(x), []interface{}(y)), nil
}

var listBinaryOperations = map[operator]func(x, y []interface{}) interface{}{
	opAdd: func(x, y []interface{}) interface{} {
		return grotskyList(append(x, y...))
	},
	opSub: func(x, y []interface{}) interface{} {
		temp := make(map[interface{}]bool)
		for _, e := range x {
			temp[e] = true
		}
		for _, e := range y {
			if val, ok := temp[e]; ok && val {
				temp[e] = false
			}
		}
		out := make([]interface{}, 0)
		for e, ok := range temp {
			if ok {
				out = append(out, e)
			}
		}
		return grotskyList(out)
	},
	opEq: func(x, y []interface{}) interface{} {
		return grotskyBool(reflect.ValueOf(x).Pointer() == reflect.ValueOf(y).Pointer())
	},
	opNeq: func(x, y []interface{}) interface{} {
		return grotskyBool(reflect.ValueOf(x).Pointer() != reflect.ValueOf(y).Pointer())
	},
}

func (l grotskyList) get(tk *token) interface{} {
	if tk.lexeme == "length" {
		return grotskyNumber(len(l))
	}
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (l grotskyList) set(name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (l grotskyList) getOperator(op operator) (operatorApply, error) {
	if apply, ok := listBinaryOperations[op]; ok {
		return func(arguments ...interface{}) (interface{}, error) {
			return applyOpToList(apply, append([]interface{}{l}, arguments...)...)
		}, nil
	}
	if op == opNeg {
		// return unique elements
		return func(arguments ...interface{}) (interface{}, error) {
			temp := make(map[interface{}]bool)
			for _, e := range l {
				temp[e] = true
			}
			out := make([]interface{}, 0)
			for e := range temp {
				out = append(out, e)
			}
			return grotskyList(out), nil
		}, nil
	}
	return nil, errUndefinedOp
}

func (l grotskyList) String() string {
	out := "["
	i := 0
	for _, val := range l {
		out += fmt.Sprintf("%s", printObj(val))
		if len(l) > 1 && i != len(l)-1 {
			out += ", "
		}
		i++
	}
	return out + "]"
}
