package internal

type stmt interface {
	accept(stmtVisitor) R
}

type stmtVisitor interface {
	visitExprStmt(stmt *exprStmt) R
	visitTryCatchStmt(stmt *tryCatchStmt) R
	visitClassicForStmt(stmt *classicForStmt) R
	visitEnhancedForStmt(stmt *enhancedForStmt) R
	visitLetStmt(stmt *letStmt) R
	visitBlockStmt(stmt *blockStmt) R
	visitWhileStmt(stmt *whileStmt) R
	visitReturnStmt(stmt *returnStmt) R
	visitBreakStmt(stmt *breakStmt) R
	visitContinueStmt(stmt *continueStmt) R
	visitIfStmt(stmt *ifStmt) R
	visitFnStmt(stmt *fnStmt) R
	visitClassStmt(stmt *classStmt) R
}

type exprStmt struct {
	last *token
	expression expr
}

func (s *exprStmt) accept(visitor stmtVisitor) R {
	return visitor.visitExprStmt(s)
}

type tryCatchStmt struct {
	tryBody stmt
	name *token
	catchBody stmt
}

func (s *tryCatchStmt) accept(visitor stmtVisitor) R {
	return visitor.visitTryCatchStmt(s)
}

type classicForStmt struct {
	keyword *token
	initializer stmt
	condition expr
	increment expr
	body stmt
}

func (s *classicForStmt) accept(visitor stmtVisitor) R {
	return visitor.visitClassicForStmt(s)
}

type enhancedForStmt struct {
	keyword *token
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
	keyword *token
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

type breakStmt struct {
	keyword *token
}

func (s *breakStmt) accept(visitor stmtVisitor) R {
	return visitor.visitBreakStmt(s)
}

type continueStmt struct {
	keyword *token
}

func (s *continueStmt) accept(visitor stmtVisitor) R {
	return visitor.visitContinueStmt(s)
}

type ifStmt struct {
	keyword *token
	condition expr
	thenBranch []stmt
	elifs []*struct{condition expr; thenBranch []stmt}
	elseBranch []stmt
}

func (s *ifStmt) accept(visitor stmtVisitor) R {
	return visitor.visitIfStmt(s)
}

type fnStmt struct {
	name *token
	params []*token
	body []stmt
}

func (s *fnStmt) accept(visitor stmtVisitor) R {
	return visitor.visitFnStmt(s)
}

type classStmt struct {
	name *token
	superclass *variableExpr
	methods []*fnStmt
	staticMethods []*fnStmt
}

func (s *classStmt) accept(visitor stmtVisitor) R {
	return visitor.visitClassStmt(s)
}


