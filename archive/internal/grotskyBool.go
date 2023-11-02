package internal

import "fmt"

type grotskyBool bool

func (c grotskyBool) get(state *interpreterState[R], tk *token) interface{} {
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (c grotskyBool) set(state *interpreterState[R], name *token, value interface{}) {
	state.runtimeErr(errReadOnly, name)
}

func (c grotskyBool) getOperator(op operator) (operatorApply, error) {
	return nil, errUndefinedOp
}

func (c grotskyBool) String() string {
	return fmt.Sprintf("%v", bool(c))
}
