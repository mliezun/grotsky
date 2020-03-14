package internal

type Expr interface {
	accept(ExprVisitor) R
}

type ExprVisitor interface {
}


