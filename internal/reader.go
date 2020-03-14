package internal

import "fmt"

//R generic type
type R interface{}

//PrintTree Prints ast
func PrintTree(state *interpreterState) {
	out := ""
	for _, stmt := range state.stmts {
		out += stmt.accept(stringVisitor{}).(string)
	}
	fmt.Println(out)
}

type stringVisitor struct{}

func (v stringVisitor) visitExprStmt(stmt stmt) R {
	exprStmt := stmt.(*exprStmt)
	return exprStmt.expression.accept(v)
}

func (v stringVisitor) visitAssignExpr(expr expr) R {
	assign := expr.(*assignExpr)
	return fmt.Sprintf("(set %s %v)", assign.name.lexeme, assign.value.accept(v))
}

func (v stringVisitor) visitBinaryExpr(expr expr) R {
	binary := expr.(*binaryExpr)
	return fmt.Sprintf("(%s %v %v)", binary.operator.lexeme, binary.left.accept(v), binary.right.accept(v))
}

func (v stringVisitor) visitCallExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitGetExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitSetExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitSuperExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitGroupingExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitLiteralExpr(expr expr) R {
	literal := expr.(*literalExpr)
	return fmt.Sprintf("%v", literal.value)
}

func (v stringVisitor) visitLogicalExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitThisExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitUnaryExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitVariableExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitFunctionExpr(expr expr) R {
	return ""
}
