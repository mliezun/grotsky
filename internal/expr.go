package internal

type expr interface {
	accept(exprVisitor) R
}

type exprVisitor interface {
	visitAssignExpr(expr expr) R
	visitBinaryExpr(expr expr) R
	visitCallExpr(expr expr) R
	visitGetExpr(expr expr) R
	visitSetExpr(expr expr) R
	visitSuperExpr(expr expr) R
	visitGroupingExpr(expr expr) R
	visitLiteralExpr(expr expr) R
	visitLogicalExpr(expr expr) R
	visitThisExpr(expr expr) R
	visitUnaryExpr(expr expr) R
	visitVariableExpr(expr expr) R
	visitFunctionExpr(expr expr) R
}

type assignExpr struct {
	name *token
	value expr
}

func (s *assignExpr) accept(visitor exprVisitor) R {
	return visitor.visitAssignExpr(s)
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


