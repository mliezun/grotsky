package internal

type stmt interface {
	accept(stmtVisitor) R
}

type stmtVisitor interface {
	visitExprStmt(stmt stmt) R
}

type exprStmt struct {
	expression expr
}

func (s *exprStmt) accept(visitor stmtVisitor) R {
	return visitor.visitExprStmt(s)
}


