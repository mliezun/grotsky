package internal

import (
	"fmt"
	"math"
)

type grotskyNumber float64

func applyOpToNums(op func(x, y float64) interface{}, arguments ...interface{}) (interface{}, error) {
	x := arguments[0].(grotskyNumber)
	y, ok := arguments[1].(grotskyNumber)
	if !ok {
		return nil, errExpectedNumber
	}
	return op(float64(x), float64(y)), nil
}

var numberBinaryOperations = map[operator]func(x, y float64) interface{}{
	opAdd: func(x, y float64) interface{} {
		return grotskyNumber(x + y)
	},
	opSub: func(x, y float64) interface{} {
		return grotskyNumber(x - y)
	},
	opDiv: func(x, y float64) interface{} {
		return grotskyNumber(x / y)
	},
	opMul: func(x, y float64) interface{} {
		return grotskyNumber(x * y)
	},
	opPow: func(x, y float64) interface{} {
		return grotskyNumber(math.Pow(x, y))
	},
	opEq: func(x, y float64) interface{} {
		return grotskyBool(x == y)
	},
	opNeq: func(x, y float64) interface{} {
		return grotskyBool(x != y)
	},
	opLt: func(x, y float64) interface{} {
		return grotskyBool(x < y)
	},
	opLte: func(x, y float64) interface{} {
		return grotskyBool(x <= y)
	},
	opGt: func(x, y float64) interface{} {
		return grotskyBool(x > y)
	},
	opGte: func(x, y float64) interface{} {
		return grotskyBool(x >= y)
	},
}

func (n grotskyNumber) get(tk *token) interface{} {
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (n grotskyNumber) set(name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (n grotskyNumber) getOperator(op operator) (operatorApply, error) {
	if apply, ok := numberBinaryOperations[op]; ok {
		return func(arguments ...interface{}) (interface{}, error) {
			return applyOpToNums(apply, append([]interface{}{n}, arguments...)...)
		}, nil
	}
	if op == opNeg {
		return func(arguments ...interface{}) (interface{}, error) {
			return grotskyNumber(-n), nil
		}, nil
	}
	return nil, errUndefinedOp
}

func (n grotskyNumber) String() string {
	return fmt.Sprintf("%v", float64(n))
}
