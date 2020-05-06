package internal

type operator string

const (
	opAdd operator = "add"
	opSub          = "sub"
	opDiv          = "div"
	opMul          = "mul"
	opPow          = "pow"
	opNeg          = "neg"
	opEq           = "eq"
	opNeq          = "neq"
	opLt           = "lt"
	opLte          = "lte"
	opGt           = "gt"
	opGte          = "gte"
	opAcc          = "acc"
)

type operatorApply func(arguments ...interface{}) (interface{}, error)
