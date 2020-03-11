package internal

type R interface {
	Read(interface{})
}

type StmtVisitor interface {
	visitExpressionStmt(stmt StmtExpr) R
}

type ExprVisitor interface {
}

type Stmt interface {
	accept(StmtVisitor) R
}

type Expr interface {
	accept(ExprVisitor) R
}

type StmtExpr struct {
	expression Expr
}

func (s StmtExpr) accept(visitor StmtVisitor) R {
	return visitor.visitExpressionStmt(s)
}
