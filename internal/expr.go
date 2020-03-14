package internal

type Expr interface {
	accept(ExprVisitor) R
}

type ExprVisitor interface {
	visitAssignExpr(expr Expr) R
	visitBinaryExpr(expr Expr) R
	visitCallExpr(expr Expr) R
	visitGetExpr(expr Expr) R
	visitSetExpr(expr Expr) R
	visitSuperExpr(expr Expr) R
	visitGroupingExpr(expr Expr) R
	visitLiteralExpr(expr Expr) R
	visitLogicalExpr(expr Expr) R
	visitThisExpr(expr Expr) R
	visitUnaryExpr(expr Expr) R
	visitVariableExpr(expr Expr) R
	visitFunctionExpr(expr Expr) R
}

type AssignExpr struct {
	name *Token
	value Expr
}

func (s *AssignExpr) accept(visitor ExprVisitor) R {
	return visitor.visitAssignExpr(s)
}

type BinaryExpr struct {
	left Expr
	operator *Token
	right Expr
}

func (s *BinaryExpr) accept(visitor ExprVisitor) R {
	return visitor.visitBinaryExpr(s)
}

type CallExpr struct {
	callee Expr
	paren *Token
	arguments []Expr
}

func (s *CallExpr) accept(visitor ExprVisitor) R {
	return visitor.visitCallExpr(s)
}

type GetExpr struct {
	object Expr
	name *Token
}

func (s *GetExpr) accept(visitor ExprVisitor) R {
	return visitor.visitGetExpr(s)
}

type SetExpr struct {
	object Expr
	name *Token
	value Expr
}

func (s *SetExpr) accept(visitor ExprVisitor) R {
	return visitor.visitSetExpr(s)
}

type SuperExpr struct {
	keyword *Token
	method *Token
}

func (s *SuperExpr) accept(visitor ExprVisitor) R {
	return visitor.visitSuperExpr(s)
}

type GroupingExpr struct {
	expression Expr
}

func (s *GroupingExpr) accept(visitor ExprVisitor) R {
	return visitor.visitGroupingExpr(s)
}

type LiteralExpr struct {
	value interface{}
}

func (s *LiteralExpr) accept(visitor ExprVisitor) R {
	return visitor.visitLiteralExpr(s)
}

type LogicalExpr struct {
	left Expr
	operator *Token
	right Expr
}

func (s *LogicalExpr) accept(visitor ExprVisitor) R {
	return visitor.visitLogicalExpr(s)
}

type ThisExpr struct {
	keyword *Token
}

func (s *ThisExpr) accept(visitor ExprVisitor) R {
	return visitor.visitThisExpr(s)
}

type UnaryExpr struct {
	operator *Token
	right Expr
}

func (s *UnaryExpr) accept(visitor ExprVisitor) R {
	return visitor.visitUnaryExpr(s)
}

type VariableExpr struct {
	name *Token
}

func (s *VariableExpr) accept(visitor ExprVisitor) R {
	return visitor.visitVariableExpr(s)
}

type FunctionExpr struct {
	params []*Token
	body []Stmt
}

func (s *FunctionExpr) accept(visitor ExprVisitor) R {
	return visitor.visitFunctionExpr(s)
}


