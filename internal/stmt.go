package internal

type stmt[T any] interface {
	accept(stmtVisitor[T]) T
}

type stmtVisitor[T any] interface {
	visitExprStmt(stmt *exprStmt[T]) T
	visitTryCatchStmt(stmt *tryCatchStmt[T]) T
	visitClassicForStmt(stmt *classicForStmt[T]) T
	visitEnhancedForStmt(stmt *enhancedForStmt[T]) T
	visitLetStmt(stmt *letStmt[T]) T
	visitBlockStmt(stmt *blockStmt[T]) T
	visitWhileStmt(stmt *whileStmt[T]) T
	visitReturnStmt(stmt *returnStmt[T]) T
	visitBreakStmt(stmt *breakStmt[T]) T
	visitContinueStmt(stmt *continueStmt[T]) T
	visitIfStmt(stmt *ifStmt[T]) T
	visitFnStmt(stmt *fnStmt[T]) T
	visitClassStmt(stmt *classStmt[T]) T
}

type exprStmt[T any] struct {
	last *token
	expression expr[T]
}

func (s *exprStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitExprStmt(s)
}

type tryCatchStmt[T any] struct {
	tryBody stmt[T]
	name *token
	catchBody stmt[T]
}

func (s *tryCatchStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitTryCatchStmt(s)
}

type classicForStmt[T any] struct {
	keyword *token
	initializer stmt[T]
	condition expr[T]
	increment expr[T]
	body stmt[T]
}

func (s *classicForStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitClassicForStmt(s)
}

type enhancedForStmt[T any] struct {
	keyword *token
	identifiers []*token
	collection expr[T]
	body stmt[T]
}

func (s *enhancedForStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitEnhancedForStmt(s)
}

type letStmt[T any] struct {
	name *token
	initializer expr[T]
}

func (s *letStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitLetStmt(s)
}

type blockStmt[T any] struct {
	stmts []stmt[T]
}

func (s *blockStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitBlockStmt(s)
}

type whileStmt[T any] struct {
	keyword *token
	condition expr[T]
	body stmt[T]
}

func (s *whileStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitWhileStmt(s)
}

type returnStmt[T any] struct {
	keyword *token
	value expr[T]
}

func (s *returnStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitReturnStmt(s)
}

type breakStmt[T any] struct {
	keyword *token
}

func (s *breakStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitBreakStmt(s)
}

type continueStmt[T any] struct {
	keyword *token
}

func (s *continueStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitContinueStmt(s)
}

type ifStmt[T any] struct {
	keyword *token
	condition expr[T]
	thenBranch []stmt[T]
	elifs []*struct{condition expr[T]; thenBranch []stmt[T]}
	elseBranch []stmt[T]
}

func (s *ifStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitIfStmt(s)
}

type fnStmt[T any] struct {
	name *token
	params []*token
	body []stmt[T]
}

func (s *fnStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitFnStmt(s)
}

type classStmt[T any] struct {
	name *token
	superclass *variableExpr[T]
	methods []*fnStmt[T]
	staticMethods []*fnStmt[T]
}

func (s *classStmt[T]) accept(visitor stmtVisitor[T]) T {
	return visitor.visitClassStmt(s)
}


