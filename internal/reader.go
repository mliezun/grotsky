package internal

import "fmt"

//R generic type
type R interface{}

//PrintTree Prints ast
func (state *state) PrintTree() {
	out := ""
	for _, stmt := range state.stmts {
		out += stmt.accept(stringVisitor{}).(string) + "\n"
	}
	fmt.Print(out)
}

type stringVisitor struct{}

func (v stringVisitor) visitExprStmt(stmt *exprStmt) R {
	return stmt.expression.accept(v)
}

func (v stringVisitor) visitFnStmt(stmt *fnStmt) R {
	out := "(fn " + stmt.name.lexeme + " ("
	for i, param := range stmt.params {
		out += param.lexeme
		if i < len(stmt.params)-1 {
			out += ", "
		}
	}
	out += ")"
	for _, st := range stmt.body {
		out += fmt.Sprintf(" %v", st.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitClassicForStmt(stmt *classicForStmt) R {
	return fmt.Sprintf(
		"(for %v %v %v %v)",
		stmt.initializer.accept(v),
		stmt.condition.accept(v),
		stmt.increment.accept(v),
		stmt.body.accept(v),
	)
}

func (v stringVisitor) visitEnhancedForStmt(stmt *enhancedForStmt) R {
	out := "(for (in ("
	for i, id := range stmt.identifiers {
		out += id.lexeme
		if i < len(stmt.identifiers)-1 {
			out += ", "
		}
	}
	out += fmt.Sprintf(") %v) %v)", stmt.collection.accept(v), stmt.body.accept(v))
	return out
}

func (v stringVisitor) visitLetStmt(stmt *letStmt) R {
	return fmt.Sprintf("(let %s %v)", stmt.name.lexeme, stmt.initializer.accept(v))
}

func (v stringVisitor) visitBlockStmt(stmt *blockStmt) R {
	out := "(scope"
	for _, s := range stmt.stmts {
		out += fmt.Sprintf(" %v", s.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitWhileStmt(stmt *whileStmt) R {
	return fmt.Sprintf("(while %v %v)", stmt.condition.accept(v), stmt.body.accept(v))
}

func (v stringVisitor) visitReturnStmt(stmt *returnStmt) R {
	return fmt.Sprintf("(return %v)", stmt.value.accept(v))
}

func (v stringVisitor) printArray(stmts []stmt) string {
	out := ""
	for _, s := range stmts {
		out += fmt.Sprintf(" %v", s.accept(v))
	}
	return out
}

func (v stringVisitor) visitIfStmt(stmt *ifStmt) R {
	out := fmt.Sprintf("(if (then %v%v)", stmt.condition.accept(v), v.printArray(stmt.thenBranch))

	for _, elif := range stmt.elifs {
		out += fmt.Sprintf(" (elif %v %v)", elif.condition.accept(v), v.printArray(elif.thenBranch))
	}
	if stmt.elseBranch != nil {
		out += fmt.Sprintf(" (else %v)", v.printArray(stmt.elseBranch))
	}
	return out + ")"
}

func (v stringVisitor) visitListExpr(expr *listExpr) R {
	out := "(list"
	for _, el := range expr.elements {
		out += fmt.Sprintf(" %v", el.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitDictionaryExpr(expr *dictionaryExpr) R {
	out := "(dict"
	for i := 0; i < len(expr.elements)/2; i++ {
		key := expr.elements[i*2]
		value := expr.elements[i*2+1]
		out += fmt.Sprintf(" %v=>%v", key.accept(v), value.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitAssignExpr(expr *assignExpr) R {
	return fmt.Sprintf("(set %s %v)", expr.name.lexeme, expr.value.accept(v))
}

func (v stringVisitor) visitAccessExpr(expr *accessExpr) R {
	slice := "#"
	if expr.first != nil {
		slice += fmt.Sprintf("%v", expr.first.accept(v))
	}
	if expr.firstColon != nil {
		slice += ":"
	}
	if expr.second != nil {
		slice += fmt.Sprintf("%v", expr.second.accept(v))
	}
	if expr.secondColon != nil {
		slice += ":"
	}
	if expr.third != nil {
		slice += fmt.Sprintf("%v", expr.third.accept(v))
	}
	return fmt.Sprintf("(%v %v)", slice, expr.object.accept(v))
}

func (v stringVisitor) visitBinaryExpr(expr *binaryExpr) R {
	return fmt.Sprintf("(%s %v %v)", expr.operator.lexeme, expr.left.accept(v), expr.right.accept(v))
}

func (v stringVisitor) visitCallExpr(expr *callExpr) R {
	return ""
}

func (v stringVisitor) visitGetExpr(expr *getExpr) R {
	return ""
}

func (v stringVisitor) visitSetExpr(expr *setExpr) R {
	return ""
}

func (v stringVisitor) visitSuperExpr(expr *superExpr) R {
	return ""
}

func (v stringVisitor) visitGroupingExpr(expr *groupingExpr) R {
	return expr.expression.accept(v)
}

func (v stringVisitor) visitLiteralExpr(expr *literalExpr) R {
	stringLiteral, isString := expr.value.(string)
	if isString {
		return "\"" + stringLiteral + "\""
	}
	return fmt.Sprintf("%v", expr.value)
}

func (v stringVisitor) visitLogicalExpr(expr *logicalExpr) R {
	return fmt.Sprintf("(%s %v %v)", expr.operator.lexeme, expr.left.accept(v), expr.right.accept(v))
}

func (v stringVisitor) visitThisExpr(expr *thisExpr) R {
	return ""
}

func (v stringVisitor) visitUnaryExpr(expr *unaryExpr) R {
	return fmt.Sprintf("(%s %v)", expr.operator.lexeme, expr.right.accept(v))
}

func (v stringVisitor) visitVariableExpr(expr *variableExpr) R {
	return expr.name.lexeme
}

func (v stringVisitor) visitFunctionExpr(expr *functionExpr) R {
	return ""
}
