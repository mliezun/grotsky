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

func (v stringVisitor) visitFnStmt(stmt stmt) R {
	fnStmt := stmt.(*fnStmt)
	out := "(fn " + fnStmt.name.lexeme + " ("
	for i, param := range fnStmt.params {
		out += param.lexeme
		if i < len(fnStmt.params)-1 {
			out += ", "
		}
	}
	out += ")"
	for _, st := range fnStmt.body {
		out += fmt.Sprintf(" %v", st.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitClassicForStmt(stmt stmt) R {
	forStmt := stmt.(*classicForStmt)
	return fmt.Sprintf(
		"(for %v %v %v %v)",
		forStmt.initializer.accept(v),
		forStmt.condition.accept(v),
		forStmt.increment.accept(v),
		forStmt.body.accept(v),
	)
}

func (v stringVisitor) visitEnhancedForStmt(stmt stmt) R {
	forStmt := stmt.(*enhancedForStmt)
	out := "(for (in ("
	for i, id := range forStmt.identifiers {
		out += id.lexeme
		if i < len(forStmt.identifiers)-1 {
			out += ", "
		}
	}
	out += fmt.Sprintf(") %v) %v)", forStmt.collection.accept(v), forStmt.body.accept(v))
	return out
}

func (v stringVisitor) visitLetStmt(stmt stmt) R {
	letStmt := stmt.(*letStmt)
	return fmt.Sprintf("(let %s %v)", letStmt.name.lexeme, letStmt.initializer.accept(v))
}

func (v stringVisitor) visitBlockStmt(stmt stmt) R {
	blockStmt := stmt.(*blockStmt)
	out := "(scope"
	for _, s := range blockStmt.stmts {
		out += fmt.Sprintf(" %v", s.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitWhileStmt(stmt stmt) R {
	whileStmt := stmt.(*whileStmt)
	return fmt.Sprintf("(while %v %v)", whileStmt.condition.accept(v), whileStmt.body.accept(v))
}

func (v stringVisitor) visitReturnStmt(stmt stmt) R {
	returnStmt := stmt.(*returnStmt)
	return fmt.Sprintf("(return %v)", returnStmt.value.accept(v))
}

func (v stringVisitor) visitIfStmt(stmt stmt) R {
	ifStmt := stmt.(*ifStmt)
	out := fmt.Sprintf("(if (then %v %v)", ifStmt.condition.accept(v), ifStmt.thenBranch.accept(v))
	for _, elif := range ifStmt.elifs {
		out += fmt.Sprintf(" %v", elif.accept(v))
	}
	if ifStmt.elseBranch != nil {
		out += fmt.Sprintf(" (else %v)", ifStmt.elseBranch.accept(v))
	}
	return out + ")"
}

func (v stringVisitor) visitElifStmt(stmt stmt) R {
	elifStmt := stmt.(*elifStmt)
	return fmt.Sprintf("(elif %v %v)", elifStmt.condition.accept(v), elifStmt.body.accept(v))
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

func (v stringVisitor) visitAccessExpr(expr expr) R {
	access := expr.(*accessExpr)
	return fmt.Sprintf("(%v %v)", access.slice.accept(v), access.object.accept(v))
}

func (v stringVisitor) visitSliceExpr(expr expr) R {
	slice := expr.(*sliceExpr)
	out := "#"
	if slice.first != nil {
		out += fmt.Sprintf("%v", slice.first.accept(v))
	}
	if slice.firstColon != nil {
		out += ":"
	}
	if slice.second != nil {
		out += fmt.Sprintf("%v", slice.second.accept(v))
	}
	if slice.secondColon != nil {
		out += ":"
	}
	if slice.third != nil {
		out += fmt.Sprintf("%v", slice.third.accept(v))
	}
	return out
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
	group := expr.(*groupingExpr)
	return group.expression.accept(v)
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
	logical := expr.(*logicalExpr)
	return fmt.Sprintf("(%s %v %v)", logical.operator.lexeme, logical.left.accept(v), logical.right.accept(v))
}

func (v stringVisitor) visitThisExpr(expr expr) R {
	return ""
}

func (v stringVisitor) visitUnaryExpr(expr expr) R {
	unary := expr.(*unaryExpr)
	return fmt.Sprintf("(%s %v)", unary.operator.lexeme, unary.right.accept(v))
}

func (v stringVisitor) visitVariableExpr(expr expr) R {
	variableExpr := expr.(*variableExpr)
	return variableExpr.name.lexeme
}

func (v stringVisitor) visitFunctionExpr(expr expr) R {
	return ""
}
