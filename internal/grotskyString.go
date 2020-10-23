package internal

type grotskyString string

// Representable object that can be represented as a string
type Representable interface {
	Repr() string
}

func applyOpToStrings(op func(x, y string) interface{}, arguments ...interface{}) (interface{}, error) {
	x := arguments[0].(grotskyString)
	y, ok := arguments[1].(grotskyString)
	if !ok {
		return nil, errExpectedString
	}
	return op(string(x), string(y)), nil
}

var stringBinaryOperations = map[operator]func(x, y string) interface{}{
	opAdd: func(x, y string) interface{} {
		return grotskyString(x + y)
	},
	opEq: func(x, y string) interface{} {
		return x == y
	},
	opNeq: func(x, y string) interface{} {
		return x != y
	},
	opGt: func(x, y string) interface{} {
		return x > y
	},
	opGte: func(x, y string) interface{} {
		return x >= y
	},
	opLt: func(x, y string) interface{} {
		return x < y
	},
	opLte: func(x, y string) interface{} {
		return x <= y
	},
}

func (s grotskyString) get(tk *token) interface{} {
	if tk.lexeme == "length" {
		return grotskyNumber(len(s))
	}

	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (s grotskyString) set(name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (s grotskyString) getOperator(op operator) (operatorApply, error) {
	if apply, ok := stringBinaryOperations[op]; ok {
		return func(arguments ...interface{}) (interface{}, error) {
			return applyOpToStrings(apply, append([]interface{}{s}, arguments...)...)
		}, nil
	}
	return nil, errUndefinedOp
}

func (s grotskyString) String() string {
	return string(s)
}

func (s grotskyString) Repr() string {
	return "\"" + string(s) + "\""
}
