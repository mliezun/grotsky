package internal

type grotskyNumber float64

var numberOperations = map[operator]operatorApply{
	opAdd: func(arguments ...interface{}) interface{} {
		n1, ok := arguments[0].(grotskyNumber)
		if !ok {
			// TODO: handle error
		}
		n2, ok := arguments[1].(grotskyNumber)
		if !ok {
			// TODO: handle error
		}
		return n1 + n2
	},
	opSub: func(arguments ...interface{}) interface{} {
		n1, ok := arguments[0].(grotskyNumber)
		if !ok {
			// TODO: handle error
		}
		n2, ok := arguments[1].(grotskyNumber)
		if !ok {
			// TODO: handle error
		}
		return n1 - n2
	},
}

func (n grotskyNumber) get(tk *token) interface{} {
	return nil
}

func (n grotskyNumber) set(name *token, value interface{}) {
}

func (n grotskyNumber) getOperator(op operator) operatorApply {
	if apply, ok := numberOperations[op]; ok {
		return apply
	}
	// TODO: handle error
	return nil
}
