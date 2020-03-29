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
		if result := e.execute(s); result != nil {
			fmt.Printf("%v\n", result)
		}
	}
}

func (e *exec) execute(stmt stmt) R {
	return stmt.accept(e)
}

func (e *exec) visitExprStmt(stmt stmt) R {
	exprStmt := stmt.(*exprStmt)
	return exprStmt.expression.accept(e)
}

func (e *exec) visitClassicForStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitEnhancedForStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitLetStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitBlockStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitWhileStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitReturnStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitIfStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitElifStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitFnStmt(stmt stmt) R {
	//TODO: implement
	return nil
}

func (e *exec) visitListExpr(expr expr) R {
	listExpr := expr.(*listExpr)
	list := make([]interface{}, len(listExpr.elements))
	for i, el := range listExpr.elements {
		list[i] = el.accept(e)
	}
	return list
}

func (e *exec) visitDictionaryExpr(expr expr) R {
	dictExpr := expr.(*dictionaryExpr)
	dict := make(map[interface{}]interface{})
	for i := 0; i < len(dictExpr.elements)/2; i++ {
		dict[dictExpr.elements[i*2].accept(e)] = dictExpr.elements[i*2+1].accept(e)
	}
	return dict
}

func (e *exec) visitAssignExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitAccessExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitSliceExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitBinaryExpr(expr expr) R {
	binExpr := expr.(*binaryExpr)
	left := binExpr.left.accept(e)
	right := binExpr.right.accept(e)
	switch binExpr.operator.token {
	case EQUAL_EQUAL:
		return left == right
	case BANG_EQUAL:
		return left != right
	case GREATER:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum > rightNum
	case GREATER_EQUAL:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum >= rightNum
	case LESS:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum < rightNum
	case LESS_EQUAL:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum <= rightNum
	case PLUS:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum + rightNum
	case MINUS:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum - rightNum
	case SLASH:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum / rightNum
	case STAR:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return leftNum * rightNum
	case POWER:
		leftNum, rightNum := e.getNums(binExpr, left, right)
		return math.Pow(leftNum, rightNum)
	default:
		e.state.runtimeErr(errUndefinedOp, binExpr.operator)
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

func (e *exec) visitCallExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitGetExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitSetExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitSuperExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitGroupingExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitLiteralExpr(expr expr) R {
	litExpr := expr.(*literalExpr)
	return litExpr.value
}

func (e *exec) visitLogicalExpr(expr expr) R {
	logExpr := expr.(*logicalExpr)
	left := e.truthy(logExpr.left.accept(e))

	if logExpr.operator.token == OR {
		if left {
			return true
		}
		right := e.truthy(logExpr.right.accept(e))
		return left || right
	}

	if logExpr.operator.token == AND {
		if !left {
			return false
		}
		right := e.truthy(logExpr.right.accept(e))
		return left && right
	}

	e.state.runtimeErr(errUndefinedOp, logExpr.operator)

	return nil
}

func (e *exec) visitThisExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitUnaryExpr(expr expr) R {
	unaryExpr := expr.(*unaryExpr)
	value := unaryExpr.right.accept(e)
	switch unaryExpr.operator.token {
	case NOT:
		return !e.truthy(value)
	case MINUS:
		valueNum, ok := value.(float64)
		if !ok {
			e.state.runtimeErr(errOnlyNumbers, unaryExpr.operator)
		}
		return -valueNum
	default:
		e.state.runtimeErr(errUndefinedOp, unaryExpr.operator)
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

func (e *exec) visitVariableExpr(expr expr) R {
	//TODO: implement
	return nil
}

func (e *exec) visitFunctionExpr(expr expr) R {
	//TODO: implement
	return nil
}
