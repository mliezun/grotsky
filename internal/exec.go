package internal

import (
	"fmt"
	"math"
	"os"
)

type exec struct {
	state *state

	globals *env
	env     *env
}

func (e *exec) interpret() {
	defer func() {
		if r := recover(); r != nil {
			runErr := e.state.runtimeError
			fmt.Fprintf(
				os.Stderr,
				"Error on line %d\n\t%s: %s\n",
				runErr.token.line,
				runErr.err.Error(),
				runErr.token.lexeme,
			)
		}
	}()
	for _, s := range e.state.stmts {
		s.accept(e)
	}
}

func (e *exec) visitExprStmt(stmt *exprStmt) R {
	return stmt.expression.accept(e)
}

func (e *exec) visitClassicForStmt(stmt *classicForStmt) R {
	for stmt.initializer.accept(e); e.truthy(stmt.condition.accept(e)); stmt.increment.accept(e) {
		stmt.body.accept(e)
	}
	return nil
}

func (e *exec) visitEnhancedForStmt(stmt *enhancedForStmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitLetStmt(stmt *letStmt) R {
	var val interface{}
	if stmt.initializer != nil {
		val = stmt.initializer.accept(e)
	}
	e.env.define(stmt.name.lexeme, val)
	return nil
}

func (e *exec) visitBlockStmt(stmt *blockStmt) R {
	e.executeBlock(stmt.stmts, newEnv(e.state, e.env))
	return nil
}

func (e *exec) executeBlock(stmts []stmt, env *env) {
	previous := e.env
	defer func() {
		e.env = previous
	}()
	e.env = env
	for _, s := range stmts {
		s.accept(e)
	}
}

func (e *exec) visitWhileStmt(stmt *whileStmt) R {
	for e.truthy(stmt.condition.accept(e)) {
		stmt.body.accept(e)
	}
	return nil
}

func (e *exec) visitReturnStmt(stmt *returnStmt) R {
	if stmt.value != nil {
		panic(returnValue(stmt.value.accept(e)))
	}
	return nil
}

func (e *exec) visitIfStmt(stmt *ifStmt) R {
	if e.truthy(stmt.condition.accept(e)) {
		return stmt.thenBranch.accept(e)
	}
	for _, elif := range stmt.elifs {
		if e.truthy(elif.accept(e)) {
			return nil
		}
	}
	if stmt.elseBranch != nil {
		return stmt.elseBranch.accept(e)
	}
	return nil
}

func (e *exec) visitElifStmt(stmt *elifStmt) R {
	if e.truthy(stmt.condition.accept(e)) {
		stmt.body.accept(e)
		return true
	}
	return nil
}

func (e *exec) visitFnStmt(stmt *fnStmt) R {
	e.env.define(stmt.name.lexeme, &function{
		declaration:   stmt,
		closure:       e.env,
		isInitializer: false,
	})
	return nil
}

func (e *exec) visitListExpr(expr *listExpr) R {
	list := make([]interface{}, len(expr.elements))
	for i, el := range expr.elements {
		list[i] = el.accept(e)
	}
	return list
}

func (e *exec) visitDictionaryExpr(expr *dictionaryExpr) R {
	dict := make(map[interface{}]interface{})
	for i := 0; i < len(expr.elements)/2; i++ {
		dict[expr.elements[i*2].accept(e)] = expr.elements[i*2+1].accept(e)
	}
	return dict
}

func (e *exec) visitAssignExpr(expr *assignExpr) R {
	val := expr.value.accept(e)
	e.env.assign(expr.name, val)
	return val
}

func (e *exec) visitAccessExpr(expr *accessExpr) R {
	object := expr.object.accept(e)
	list, isList := object.([]interface{})
	if isList {
		return e.sliceList(list, expr)
	}
	dict, isDict := object.(map[interface{}]interface{})
	if isDict {
		if expr.first == nil {
			e.state.runtimeErr(errExpectedKey, expr.brace)
		}
		return dict[expr.first.accept(e)]
	}
	e.state.runtimeErr(errInvalidAccess, expr.brace)
	return nil
}

func (e *exec) sliceList(list []interface{}, accessExpr *accessExpr) interface{} {
	first, second, third := e.exprToInt(accessExpr.first, accessExpr.brace),
		e.exprToInt(accessExpr.second, accessExpr.brace),
		e.exprToInt(accessExpr.third, accessExpr.brace)

	if first != nil {
		if accessExpr.firstColon != nil {
			if second != nil {
				// [a:b:c]
				if accessExpr.secondColon != nil {
					if third == nil {
						e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
					}
					return e.stepList(list[*first:*second], *third)
				}
				// [a:b]
				return list[*first:*second]
			}

			// [a::c]
			if accessExpr.secondColon != nil {
				if third == nil {
					e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
				}
				return e.stepList(list[*first:], *third)
			}

			// [a:]
			return list[*first:]
		}
		// [a]
		return list[*first]
	}

	if second != nil {
		// [:b:c]
		if accessExpr.secondColon != nil {
			if third == nil {
				e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
			}
			return e.stepList(list[:*second], *third)
		}
		// [:b]
		return list[:*second]
	}

	if third == nil {
		e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
	}
	// [::c]
	return e.stepList(list, *third)
}

func (e *exec) exprToInt(expr expr, token *token) *int64 {
	if expr == nil {
		return nil
	}
	valueF, ok := expr.accept(e).(float64)
	if !ok {
		e.state.runtimeErr(errOnlyNumbers, token)
	}
	valueI := int64(valueF)
	return &valueI
}

func (e *exec) stepList(list []interface{}, step int64) []interface{} {
	if step <= 1 {
		return list
	}
	out := make([]interface{}, 0)
	if step > int64(len(list)) {
		return out
	}
	for i, el := range list {
		if int64(i)%step == 0 {
			out = append(out, el)
		}
	}
	return out
}

func (e *exec) visitBinaryExpr(expr *binaryExpr) R {
	left := expr.left.accept(e)
	right := expr.right.accept(e)
	switch expr.operator.token {
	case EQUAL_EQUAL:
		return left == right
	case BANG_EQUAL:
		return left != right
	case GREATER:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum > rightNum
	case GREATER_EQUAL:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum >= rightNum
	case LESS:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum < rightNum
	case LESS_EQUAL:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum <= rightNum
	case PLUS:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum + rightNum
	case MINUS:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum - rightNum
	case SLASH:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum / rightNum
	case STAR:
		leftNum, rightNum := e.getNums(expr, left, right)
		return leftNum * rightNum
	case POWER:
		leftNum, rightNum := e.getNums(expr, left, right)
		return math.Pow(leftNum, rightNum)
	default:
		e.state.runtimeErr(errUndefinedOp, expr.operator)
	}
	return nil
}

func (e *exec) getNums(binExpr *binaryExpr, left, right interface{}) (float64, float64) {
	leftNum, ok := left.(float64)
	if !ok {
		e.state.runtimeErr(errOnlyNumbers, binExpr.operator)
	}
	rightNum, ok := right.(float64)
	if !ok {
		e.state.runtimeErr(errOnlyNumbers, binExpr.operator)
	}
	return leftNum, rightNum
}

func (e *exec) visitCallExpr(expr *callExpr) R {
	callee := expr.callee.accept(e)
	arguments := make([]interface{}, len(expr.arguments))
	for i := range expr.arguments {
		arguments[i] = expr.arguments[i].accept(e)
	}

	fn, isFn := callee.(callable)
	if !isFn {
		e.state.runtimeErr(errOnlyFunction, expr.paren)
	}

	if len(arguments) != fn.arity() {
		e.state.runtimeErr(errInvalidNumberArguments, expr.paren)
	}

	return fn.call(e, arguments)
}

func (e *exec) visitGetExpr(expr *getExpr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitSetExpr(expr *setExpr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitSuperExpr(expr *superExpr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitGroupingExpr(expr *groupingExpr) R {
	return expr.expression.accept(e)
}

func (e *exec) visitLiteralExpr(expr *literalExpr) R {
	return expr.value
}

func (e *exec) visitLogicalExpr(expr *logicalExpr) R {
	left := e.truthy(expr.left.accept(e))

	if expr.operator.token == OR {
		if left {
			return true
		}
		right := e.truthy(expr.right.accept(e))
		return left || right
	}

	if expr.operator.token == AND {
		if !left {
			return false
		}
		right := e.truthy(expr.right.accept(e))
		return left && right
	}

	e.state.runtimeErr(errUndefinedOp, expr.operator)

	return nil
}

func (e *exec) visitThisExpr(expr *thisExpr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitUnaryExpr(expr *unaryExpr) R {
	value := expr.right.accept(e)
	switch expr.operator.token {
	case NOT:
		return !e.truthy(value)
	case MINUS:
		valueNum, ok := value.(float64)
		if !ok {
			e.state.runtimeErr(errOnlyNumbers, expr.operator)
		}
		return -valueNum
	default:
		e.state.runtimeErr(errUndefinedOp, expr.operator)
	}
	return nil
}

func (e *exec) truthy(value interface{}) bool {
	if value == nil {
		return false
	}
	valueStr, isStr := value.(string)
	if isStr {
		return valueStr != ""
	}
	valueNum, isNum := value.(float64)
	if isNum {
		return valueNum != 0
	}
	valueBool, isBool := value.(bool)
	if isBool {
		return valueBool
	}
	return true
}

func (e *exec) visitVariableExpr(expr *variableExpr) R {
	return e.env.get(expr.name)
}

func (e *exec) visitFunctionExpr(expr *functionExpr) R {
	//TODO: implement
	return nil
}
