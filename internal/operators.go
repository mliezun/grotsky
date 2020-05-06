package internal

type operator string

const (
	opAdd operator = "add"
	opSub          = "sub"
	opDiv          = "div"
	opMul          = "mul"
	opPow          = "pow"
	opNeg          = "negate"
	opAnd          = "and"
	opOr           = "or"
	opNot          = "not"
	opEq           = "eq"
	opNeq          = "neq"
	opLt           = "lt"
	opLte          = "lte"
	opGt           = "gt"
	opGte          = "gte"
	opAcc          = "acc"
)

type operatorApply func(arguments ...interface{}) (interface{}, error)

func makeOperatorApplier(obj interface{}, apply operatorApply) operatorApply {
	return func(arguments ...interface{}) (interface{}, error) {
		return apply(append([]interface{}{obj}, arguments...)...)
	}
}
