package internal

type stmt interface {
	accept(stmtVisitor) R
}

type stmtVisitor interface {
	visitExprStmt(stmt stmt) R
	visitClassicForStmt(stmt stmt) R
	visitEnhancedForStmt(stmt stmt) R
	visitLetStmt(stmt stmt) R
	visitBlockStmt(stmt stmt) R
	visitWhileStmt(stmt stmt) R
	visitReturnStmt(stmt stmt) R
	visitIfStmt(stmt stmt) R
	visitElifStmt(stmt stmt) R
	visitFnStmt(stmt stmt) R
}

type exprStmt struct {
	expression expr
}

func (s *exprStmt) accept(visitor stmtVisitor) R {
	return visitor.visitExprStmt(s)
}

type classicForStmt struct {
	initializer stmt
	condition expr
	increment expr
	body stmt
}

func (s *classicForStmt) accept(visitor stmtVisitor) R {
	return visitor.visitClassicForStmt(s)
}

type enhancedForStmt struct {
	identifiers []*token
	collection expr
	body stmt
}

func (s *enhancedForStmt) accept(visitor stmtVisitor) R {
	return visitor.visitEnhancedForStmt(s)
}

type letStmt struct {
	name *token
	initializer expr
}

func (s *letStmt) accept(visitor stmtVisitor) R {
	return visitor.visitLetStmt(s)
}

type blockStmt struct {
	stmts []stmt
}

func (s *blockStmt) accept(visitor stmtVisitor) R {
	return visitor.visitBlockStmt(s)
}

type whileStmt struct {
	condition expr
	body stmt
}

func (s *whileStmt) accept(visitor stmtVisitor) R {
	return visitor.visitWhileStmt(s)
}

type returnStmt struct {
	keyword *token
	value expr
}

func (s *returnStmt) accept(visitor stmtVisitor) R {
	return visitor.visitReturnStmt(s)
}

type ifStmt struct {
	condition expr
	thenBranch stmt
	elifs []*elifStmt
	elseBranch stmt
}

func (s *ifStmt) accept(visitor stmtVisitor) R {
	return visitor.visitIfStmt(s)
}

type elifStmt struct {
	condition expr
	body stmt
}

func (s *elifStmt) accept(visitor stmtVisitor) R {
	return visitor.visitElifStmt(s)
}

type fnStmt struct {
	name *token
	params []*token
	body []stmt
}

func (s *fnStmt) accept(visitor stmtVisitor) R {
	return visitor.visitFnStmt(s)
}


