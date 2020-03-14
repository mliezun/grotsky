package internal

import "fmt"

//R generic type
type R interface{}

//PrintTree Prints ast
func (state *interpreterState) PrintTree() {
	out := ""
	for _, stmt := range state.stmts {
		out += stmt.accept(stringVisitor{}).(string) + "\n"
	}
	fmt.Print(out)
}

type stringVisitor struct{}

func (v stringVisitor) visitExprStmt(stmt stmt) R {
	exprStmt := stmt.(*exprStmt)
	return exprStmt.expression.accept(v)
}

func (v stringVisitor) visitListExpr(expr expr) R {
	list := expr.(*listExpr)
	out := "(list"
	for _, el := range list.elements {
		out += fmt.Sprintf(" %v", el.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitDictionaryExpr(expr expr) R {
	dict := expr.(*dictionaryExpr)
	out := "(dict"
	for i := 0; i < len(dict.elements)/2; i++ {
		key := dict.elements[i*2]
		value := dict.elements[i*2+1]
		out += fmt.Sprintf(" %v=>%v", key.accept(v), value.accept(v))
	}
	return out + ")"
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
	stringLiteral, isString := literal.value.(string)
	if isString {
		return "\"" + stringLiteral + "\""
	}
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
