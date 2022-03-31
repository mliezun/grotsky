package internal

type expr[T any] interface {
	accept(exprVisitor[T]) T
}

type exprVisitor[T any] interface {
	visitListExpr(expr *listExpr[T]) T
	visitDictionaryExpr(expr *dictionaryExpr[T]) T
	visitAssignExpr(expr *assignExpr[T]) T
	visitAccessExpr(expr *accessExpr[T]) T
	visitBinaryExpr(expr *binaryExpr[T]) T
	visitCallExpr(expr *callExpr[T]) T
	visitGetExpr(expr *getExpr[T]) T
	visitSetExpr(expr *setExpr[T]) T
	visitSuperExpr(expr *superExpr[T]) T
	visitGroupingExpr(expr *groupingExpr[T]) T
	visitLiteralExpr(expr *literalExpr[T]) T
	visitLogicalExpr(expr *logicalExpr[T]) T
	visitThisExpr(expr *thisExpr[T]) T
	visitUnaryExpr(expr *unaryExpr[T]) T
	visitVariableExpr(expr *variableExpr[T]) T
	visitFunctionExpr(expr *functionExpr[T]) T
}

type listExpr[T any] struct {
	elements []expr[T]
	brace *token
}

func (s *listExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitListExpr(s)
}

type dictionaryExpr[T any] struct {
	elements []expr[T]
	curlyBrace *token
}

func (s *dictionaryExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitDictionaryExpr(s)
}

type assignExpr[T any] struct {
	name *token
	value expr[T]
	access expr[T]
}

func (s *assignExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitAssignExpr(s)
}

type accessExpr[T any] struct {
	object expr[T]
	brace *token
	first expr[T]
	firstColon *token
	second expr[T]
	secondColon *token
	third expr[T]
}

func (s *accessExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitAccessExpr(s)
}

type binaryExpr[T any] struct {
	left expr[T]
	operator *token
	right expr[T]
}

func (s *binaryExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitBinaryExpr(s)
}

type callExpr[T any] struct {
	callee expr[T]
	paren *token
	arguments []expr[T]
}

func (s *callExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitCallExpr(s)
}

type getExpr[T any] struct {
	object expr[T]
	name *token
}

func (s *getExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitGetExpr(s)
}

type setExpr[T any] struct {
	object expr[T]
	name *token
	value expr[T]
	access expr[T]
}

func (s *setExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitSetExpr(s)
}

type superExpr[T any] struct {
	keyword *token
	method *token
}

func (s *superExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitSuperExpr(s)
}

type groupingExpr[T any] struct {
	expression expr[T]
}

func (s *groupingExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitGroupingExpr(s)
}

type literalExpr[T any] struct {
	value interface{}
}

func (s *literalExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitLiteralExpr(s)
}

type logicalExpr[T any] struct {
	left expr[T]
	operator *token
	right expr[T]
}

func (s *logicalExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitLogicalExpr(s)
}

type thisExpr[T any] struct {
	keyword *token
}

func (s *thisExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitThisExpr(s)
}

type unaryExpr[T any] struct {
	operator *token
	right expr[T]
}

func (s *unaryExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitUnaryExpr(s)
}

type variableExpr[T any] struct {
	name *token
}

func (s *variableExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitVariableExpr(s)
}

type functionExpr[T any] struct {
	params []*token
	body []stmt[T]
}

func (s *functionExpr[T]) accept(visitor exprVisitor[T]) T {
	return visitor.visitFunctionExpr(s)
}


