package internal

type stmt interface {
	accept(stmtVisitor) R
}

type stmtVisitor interface {
	visitExprStmt(stmt *exprStmt) R
	visitClassicForStmt(stmt *classicForStmt) R
	visitEnhancedForStmt(stmt *enhancedForStmt) R
	visitLetStmt(stmt *letStmt) R
	visitBlockStmt(stmt *blockStmt) R
	visitWhileStmt(stmt *whileStmt) R
	visitReturnStmt(stmt *returnStmt) R
	visitIfStmt(stmt *ifStmt) R
	visitElifStmt(stmt *elifStmt) R
	visitFnStmt(stmt *fnStmt) R
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

