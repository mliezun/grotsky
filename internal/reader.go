package internal

import "fmt"

type R interface{}

func PrintTree(state *InterpreterState) {
	out := ""
	for _, stmt := range state.Stmts {
		out += stmt.accept(StringVisitor{}).(string)
	}
	fmt.Println(out)
}

type StringVisitor struct{}

func (v StringVisitor) visitExprStmt(stmt Stmt) R {
	exprStmt := stmt.(*ExprStmt)
	return exprStmt.expression.accept(v)
}

func (v StringVisitor) visitAssignExpr(expr Expr) R {
	assign := expr.(*AssignExpr)
	return fmt.Sprintf("(set %s %v)", assign.name.lexeme, assign.value.accept(v))
}

func (v StringVisitor) visitBinaryExpr(expr Expr) R {
	binary := expr.(*BinaryExpr)
	return fmt.Sprintf("(%s %v %v)", binary.operator.lexeme, binary.left.accept(v), binary.right.accept(v))
}

func (v StringVisitor) visitCallExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitGetExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitSetExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitSuperExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitGroupingExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitLiteralExpr(expr Expr) R {
	literal := expr.(*LiteralExpr)
	return fmt.Sprintf("%v", literal.value)
}

func (v StringVisitor) visitLogicalExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitThisExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitUnaryExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitVariableExpr(expr Expr) R {
	return ""
}

func (v StringVisitor) visitFunctionExpr(expr Expr) R {
	return ""
}
