package internal

type StmtVisitor interface {
	visitExpressionStmt(stmt StmtExpr) interface{}
}

type ExprVisitor interface {
}

type Stmt interface {
	accept(StmtVisitor) interface{}
}

type Expr interface {
	accept(ExprVisitor) interface{}
}

type StmtExpr struct {
	expression Expr
}

func (s StmtExpr) accept(visitor StmtVisitor) interface{} {
	return visitor.visitExpressionStmt(s)
}
