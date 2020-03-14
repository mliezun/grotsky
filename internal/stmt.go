package internal

type Stmt interface {
	accept(StmtVisitor) R
}

type StmtVisitor interface {
	visitExprStmt(stmt Stmt) R
}

type ExprStmt struct {
	expression Expr
}

func (s ExprStmt) accept(visitor StmtVisitor) R {
	return visitor.visitExprStmt(s)
}


