package internal

type expr interface {
	accept(exprVisitor) R
}

type exprVisitor interface {
	visitListExpr(expr *listExpr) R
	visitDictionaryExpr(expr *dictionaryExpr) R
	visitAssignExpr(expr *assignExpr) R
	visitAccessExpr(expr *accessExpr) R
	visitBinaryExpr(expr *binaryExpr) R
	visitCallExpr(expr *callExpr) R
	visitGetExpr(expr *getExpr) R
	visitSetExpr(expr *setExpr) R
	visitSuperExpr(expr *superExpr) R
	visitGroupingExpr(expr *groupingExpr) R
	visitLiteralExpr(expr *literalExpr) R
	visitLogicalExpr(expr *logicalExpr) R
	visitThisExpr(expr *thisExpr) R
	visitUnaryExpr(expr *unaryExpr) R
	visitVariableExpr(expr *variableExpr) R
	visitFunctionExpr(expr *functionExpr) R
}

type listExpr struct {
	elements []expr
	brace *token
}

func (s *listExpr) accept(visitor exprVisitor) R {
	return visitor.visitListExpr(s)
}

type dictionaryExpr struct {
	elements []expr
	curlyBrace *token
}

func (s *dictionaryExpr) accept(visitor exprVisitor) R {
	return visitor.visitDictionaryExpr(s)
}

type assignExpr struct {
	name *token
	value expr
}

func (s *assignExpr) accept(visitor exprVisitor) R {
	return visitor.visitAssignExpr(s)
}

type accessExpr struct {
	object expr
	brace *token
	first expr
	firstColon *token
	second expr
	secondColon *token
	third expr
}

func (s *accessExpr) accept(visitor exprVisitor) R {
	return visitor.visitAccessExpr(s)
}

type binaryExpr struct {
	left expr
	operator *token
	right expr
}

func (s *binaryExpr) accept(visitor exprVisitor) R {
	return visitor.visitBinaryExpr(s)
}

type callExpr struct {
	callee expr
	paren *token
	arguments []expr
}

func (s *callExpr) accept(visitor exprVisitor) R {
	return visitor.visitCallExpr(s)
}

type getExpr struct {
	object expr
	name *token
}

func (s *getExpr) accept(visitor exprVisitor) R {
	return visitor.visitGetExpr(s)
}

type setExpr struct {
	object expr
	name *token
	value expr
}

func (s *setExpr) accept(visitor exprVisitor) R {
	return visitor.visitSetExpr(s)
}

type superExpr struct {
	keyword *token
	method *token
}

func (s *superExpr) accept(visitor exprVisitor) R {
	return visitor.visitSuperExpr(s)
}

type groupingExpr struct {
	expression expr
}

func (s *groupingExpr) accept(visitor exprVisitor) R {
	return visitor.visitGroupingExpr(s)
}

type literalExpr struct {
	value interface{}
}

func (s *literalExpr) accept(visitor exprVisitor) R {
	return visitor.visitLiteralExpr(s)
}

type logicalExpr struct {
	left expr
	operator *token
	right expr
}

func (s *logicalExpr) accept(visitor exprVisitor) R {
	return visitor.visitLogicalExpr(s)
}

type thisExpr struct {
	keyword *token
}

func (s *thisExpr) accept(visitor exprVisitor) R {
	return visitor.visitThisExpr(s)
}

type unaryExpr struct {
	operator *token
	right expr
}

func (s *unaryExpr) accept(visitor exprVisitor) R {
	return visitor.visitUnaryExpr(s)
}

type variableExpr struct {
	name *token
}

func (s *variableExpr) accept(visitor exprVisitor) R {
	return visitor.visitVariableExpr(s)
}

type functionExpr struct {
	params []*token
	body []stmt
}

func (s *functionExpr) accept(visitor exprVisitor) R {
	return visitor.visitFunctionExpr(s)
}


