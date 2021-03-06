package internal

import (
	"sync"
)

type execute struct {
	mx *sync.Mutex

	globals *env
	env     *env

	cls []*callStack

	state *interpreterState
}

func (e *execute) getExecContext() *callStack {
	return e.cls[len(e.cls)-1]
}

func (e *execute) enterFunction(name string) {
	e.cls = append(e.cls, &callStack{
		function:  name,
		loopCount: 0,
	})
}

func (e *execute) leaveFunction(name string) {
	pc := e.getExecContext()
	if pc.function != name {
		e.state.runtimeErr(errLeavingFunction, &token{})
	}
	e.cls = e.cls[:len(e.cls)-1]
}

func (e *execute) enterLoop() {
	pc := e.getExecContext()
	pc.loopCount++
}

func (e *execute) leaveLoop() {
	pc := e.getExecContext()
	pc.loopCount--
}

func (e *execute) insideLoop() bool {
	return e.getExecContext().loopCount != 0
}

var exec = execute{}

func (e *execute) interpret() (res bool) {
	e.enterFunction("")
	//defer e.leaveFunction("")
	defer func() {
		if e.state.runtimeError != nil {
			recover()
			res = false
		}
	}()
	for _, s := range e.state.stmts {
		s.accept(e)
	}
	return true
}

func (e *execute) visitExprStmt(stmt *exprStmt) R {
	return stmt.expression.accept(e)
}

func (e *execute) visitClassicForStmt(stmt *classicForStmt) R {
	if stmt.initializer != nil {
		stmt.initializer.accept(e)
	}
	e.enterLoop()
	defer e.leaveLoop()
	for ; e.truthy(stmt.condition.accept(e)); stmt.increment.accept(e) {
		val := stmt.body.accept(e)
		switch val.(type) {
		case *returnValue:
			return val
		case *breakValue:
			if e.insideLoop() {
				return nil
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		case *continueValue:
			if e.insideLoop() {
				continue
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		}
	}
	return nil
}

func (e *execute) visitEnhancedForStmt(stmt *enhancedForStmt) R {
	collection := stmt.collection.accept(e)
	environment := newEnv(e.env)

	identifiersCount := len(stmt.identifiers)

	e.enterLoop()
	defer e.leaveLoop()

	if array, ok := collection.(grotskyList); ok {
		for _, el := range array {
			if identifiersCount == 1 {
				environment.define(stmt.identifiers[0].lexeme, el)
			} else if array2, ok := el.(grotskyList); ok {
				if len(array2) != identifiersCount {
					e.state.runtimeErr(errWrongNumberOfValues, stmt.keyword)
				}
				for i, id := range stmt.identifiers {
					environment.define(id.lexeme, array2[i])
				}
			} else {
				e.state.runtimeErr(errCannotUnpack, stmt.keyword)
			}
			val := e.executeOne(stmt.body, environment)
			switch val.(type) {
			case *returnValue:
				return val
			case *breakValue:
				if e.insideLoop() {
					return nil
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			case *continueValue:
				if e.insideLoop() {
					continue
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			}
		}
	} else if dict, ok := collection.(grotskyDict); ok {
		if identifiersCount > 2 {
			e.state.runtimeErr(errExpectedIdentifiersDict, stmt.keyword)
		}
		for key, value := range dict {
			if identifiersCount == 1 {
				environment.define(stmt.identifiers[0].lexeme, key)
			} else {
				environment.define(stmt.identifiers[0].lexeme, key)
				environment.define(stmt.identifiers[1].lexeme, value)
			}
			val := e.executeOne(stmt.body, environment)
			switch val.(type) {
			case *returnValue:
				return val
			case *breakValue:
				if e.insideLoop() {
					return nil
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			case *continueValue:
				if e.insideLoop() {
					continue
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			}
		}
	} else {
		e.state.runtimeErr(errExpectedCollection, stmt.keyword)
	}

	return nil
}

func (e *execute) visitLetStmt(stmt *letStmt) R {
	var val interface{}
	if stmt.initializer != nil {
		val = stmt.initializer.accept(e)
	}
	e.env.define(stmt.name.lexeme, val)
	return nil
}

func (e *execute) visitBlockStmt(stmt *blockStmt) R {
	return e.executeBlock(stmt.stmts, newEnv(e.env))
}

func (e *execute) executeOne(st stmt, env *env) R {
	previous := e.env
	defer func() {
		e.env = previous
	}()
	e.env = env
	return st.accept(e)
}

func (e *execute) executeBlock(stmts []stmt, env *env) R {
	previous := e.env
	defer func() {
		e.env = previous
	}()
	e.env = env
	for _, s := range stmts {
		val := s.accept(e)
		switch val.(type) {
		case *returnValue:
			return val
		case *breakValue:
			if e.insideLoop() {
				return val
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		case *continueValue:
			if e.insideLoop() {
				return val
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		}
	}
	return nil
}

func (e *execute) visitWhileStmt(stmt *whileStmt) R {
	e.enterLoop()
	defer e.leaveLoop()
	for e.truthy(stmt.condition.accept(e)) {
		val := stmt.body.accept(e)
		switch val.(type) {
		case *returnValue:
			return val
		case *breakValue:
			if e.insideLoop() {
				return nil
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		case *continueValue:
			if e.insideLoop() {
				continue
			} else {
				e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
			}
		}
	}
	return nil
}

func (e *execute) visitReturnStmt(stmt *returnStmt) R {
	if stmt.value != nil {
		result := &returnValue{value: stmt.value.accept(e)}
		return result
	}
	return nil
}

func (e *execute) visitBreakStmt(stmt *breakStmt) R {
	return &breakValue{}
}

func (e *execute) visitContinueStmt(stmt *continueStmt) R {
	return &continueValue{}
}

func (e *execute) visitIfStmt(stmt *ifStmt) R {
	if e.truthy(stmt.condition.accept(e)) {
		for _, st := range stmt.thenBranch {
			val := st.accept(e)
			switch val.(type) {
			case *returnValue:
				return val
			case *breakValue:
				if e.insideLoop() {
					return val
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			case *continueValue:
				if e.insideLoop() {
					return val
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			}
		}
		return nil
	}
	for _, elif := range stmt.elifs {
		if e.truthy(elif.condition.accept(e)) {
			for _, st := range elif.thenBranch {
				val := st.accept(e)
				switch val.(type) {
				case *returnValue:
					return val
				case *breakValue:
					if e.insideLoop() {
						return val
					} else {
						e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
					}
				case *continueValue:
					if e.insideLoop() {
						return val
					} else {
						e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
					}
				}
			}
			return nil
		}
	}
	if stmt.elseBranch != nil {
		for _, st := range stmt.elseBranch {
			val := st.accept(e)
			switch val.(type) {
			case *returnValue:
				return val
			case *breakValue:
				if e.insideLoop() {
					return val
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			case *continueValue:
				if e.insideLoop() {
					return val
				} else {
					e.state.runtimeErr(errOnlyAllowedInsideLoop, &token{})
				}
			}
		}
	}
	return nil
}

func (e *execute) visitFnStmt(stmt *fnStmt) R {
	e.env.define(stmt.name.lexeme, &grotskyFunction{
		declaration:   stmt,
		closure:       e.env,
		isInitializer: false,
	})
	return nil
}

func (e *execute) visitClassStmt(stmt *classStmt) R {
	class := &grotskyClass{
		name: stmt.name.lexeme,
	}

	if stmt.superclass != nil {
		superclass, ok := stmt.superclass.accept(e).(*grotskyClass)
		if !ok {
			e.state.runtimeErr(errExpectedClass, stmt.name)
		}
		class.superclass = superclass
	}

	e.env.define(class.name, class)

	if stmt.superclass != nil {
		e.env = newEnv(e.env)
		e.env.define("super", class.superclass)
	}

	class.methods = make(map[string]*grotskyFunction)
	for _, m := range stmt.methods {
		class.methods[m.name.lexeme] = &grotskyFunction{
			declaration:   m,
			closure:       e.env,
			isInitializer: m.name.lexeme == "init",
		}
	}
	class.staticMethods = make(map[string]*grotskyFunction)
	for _, m := range stmt.staticMethods {
		class.staticMethods[m.name.lexeme] = &grotskyFunction{
			declaration:   m,
			closure:       e.env,
			isInitializer: false,
		}
	}

	if stmt.superclass != nil {
		e.env = e.env.enclosing
	}

	return nil
}

func (e *execute) visitListExpr(expr *listExpr) R {
	list := make(grotskyList, len(expr.elements))
	for i, el := range expr.elements {
		list[i] = el.accept(e)
	}
	return list
}

func (e *execute) visitDictionaryExpr(expr *dictionaryExpr) R {
	dict := make(grotskyDict)
	for i := 0; i < len(expr.elements)/2; i++ {
		dict[expr.elements[i*2].accept(e)] = expr.elements[i*2+1].accept(e)
	}
	return dict
}

func (e *execute) visitAssignExpr(expr *assignExpr) R {
	val := expr.value.accept(e)
	e.env.assign(e.state, expr.name, val)
	return val
}

func (e *execute) visitAccessExpr(expr *accessExpr) R {
	object := expr.object.accept(e)
	str, isStr := object.(grotskyString)
	if isStr {
		if len(str) == 0 {
			return str
		}
		list := make(grotskyList, len(str))
		for i, r := range str {
			list[i] = r
		}
		result := e.sliceList(list, expr)
		if runes, ok := result.(grotskyList); ok {
			newStr := ""
			for _, r := range runes {
				newStr += string(r.(rune))
			}
			return grotskyString(newStr)
		}
		return grotskyString(result.(rune))
	}
	list, isList := object.(grotskyList)
	if isList {
		if len(list) == 0 {
			return list
		}
		return e.sliceList(list, expr)
	}
	dict, isDict := object.(grotskyDict)
	if isDict {
		if expr.first == nil || expr.second != nil || expr.third != nil {
			e.state.runtimeErr(errExpectedKey, expr.brace)
		}
		if len(dict) == 0 {
			return dict
		}
		val, _ := dict[expr.first.accept(e)]
		return val
	}
	e.state.runtimeErr(errInvalidAccess, expr.brace)
	return nil
}

func (e *execute) sliceList(list grotskyList, accessExpr *accessExpr) interface{} {
	first, second, third := e.exprToInt(accessExpr.first, accessExpr.brace),
		e.exprToInt(accessExpr.second, accessExpr.brace),
		e.exprToInt(accessExpr.third, accessExpr.brace)

	if first != nil {
		if accessExpr.firstColon != nil {
			if second != nil {
				sec := *second
				if maxLen := int64(len(list)); sec > maxLen {
					sec = maxLen
				}

				// [a:b:c]
				if accessExpr.secondColon != nil {
					if third == nil {
						e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
					}
					return e.stepList(list[*first:sec], *third)
				}
				// [a:b]
				return list[*first:sec]
			}

			// [a::c]
			if accessExpr.secondColon != nil && third != nil {
				return e.stepList(list[*first:], *third)
			}

			// [a:]
			return list[*first:]
		}
		// [a]
		return list[*first]
	}

	if second != nil {
		sec := *second
		if maxLen := int64(len(list)); sec > maxLen {
			sec = maxLen
		}
		// [:b:c]
		if accessExpr.secondColon != nil && third != nil {
			return e.stepList(list[:sec], *third)
		}
		// [:b]
		return list[:sec]
	}

	// assert third != nil
	// e.state.runtimeErr(errExpectedStep, accessExpr.secondColon)
	// [::c]
	return e.stepList(list, *third)
}

func (e *execute) exprToInt(expr expr, token *token) *int64 {
	if expr == nil {
		return nil
	}
	valueF, ok := expr.accept(e).(grotskyNumber)
	if !ok {
		e.state.runtimeErr(errOnlyNumbers, token)
	}
	valueI := int64(valueF)
	return &valueI
}

func (e *execute) stepList(list grotskyList, step int64) grotskyList {
	if step <= 1 {
		return list
	}
	out := make(grotskyList, 0)
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

func (e *execute) visitBinaryExpr(expr *binaryExpr) R {
	var value interface{}
	var err error
	left := expr.left.accept(e)
	right := expr.right.accept(e)
	switch expr.operator.token {
	case tkEqualEqual:
		value, err = e.operateBinary(opEq, left, right)
	case tkBangEqual:
		value, err = e.operateBinary(opNeq, left, right)
	case tkGreater:
		value, err = e.operateBinary(opGt, left, right)
	case tkGreaterEqual:
		value, err = e.operateBinary(opGte, left, right)
	case tkLess:
		value, err = e.operateBinary(opLt, left, right)
	case tkLessEqual:
		value, err = e.operateBinary(opLte, left, right)
	case tkPlus:
		value, err = e.operateBinary(opAdd, left, right)
	case tkMinus:
		value, err = e.operateBinary(opSub, left, right)
	case tkSlash:
		value, err = e.operateBinary(opDiv, left, right)
	case tkMod:
		value, err = e.operateBinary(opMod, left, right)
	case tkStar:
		value, err = e.operateBinary(opMul, left, right)
	case tkPower:
		value, err = e.operateBinary(opPow, left, right)
	}
	if err != nil {
		e.state.runtimeErr(err, expr.operator)
	}
	return value
}

func (e *execute) operateUnary(op operator, left interface{}) (interface{}, error) {
	leftVal := left.(grotskyInstance)
	apply, err := leftVal.getOperator(op)
	if err != nil {
		return nil, err
	}
	return apply()
}

// equalingNil if at least one of right or left is nil and operator is Eq or Neq returns result
func equalingNil(op operator, left, right interface{}) (shouldCompare bool, result bool) {
	result = false
	shouldCompare = false
	if op == opEq {
		shouldCompare = true
		result = left == right
	}
	if op == opNeq {
		shouldCompare = true
		result = left != right
	}
	return
}

func (e *execute) operateBinary(op operator, left, right interface{}) (interface{}, error) {
	if left != nil && right != nil {
		leftVal := left.(grotskyInstance)
		apply, err := leftVal.getOperator(op)
		if err != nil {
			return nil, err
		}
		return apply(right)
	}
	if shouldCompare, result := equalingNil(op, left, right); shouldCompare {
		return grotskyBool(result), nil
	}
	return nil, errUndefinedOp
}

func (e *execute) visitCallExpr(expr *callExpr) R {
	callee := expr.callee.accept(e)
	arguments := make([]interface{}, len(expr.arguments))
	for i := range expr.arguments {
		arguments[i] = expr.arguments[i].accept(e)
	}

	fn, isFn := callee.(grotskyCallable)
	if !isFn {
		e.state.runtimeErr(errOnlyFunction, expr.paren)
	}

	e.enterFunction(fn.String())
	defer e.leaveFunction(fn.String())

	result, err := fn.call(arguments)
	if err != nil {
		e.state.runtimeErr(err, expr.paren)
	}

	return result
}

func (e *execute) visitGetExpr(expr *getExpr) R {
	object := expr.object.accept(e)
	if obj, ok := object.(grotskyInstance); ok {
		return obj.get(e.state, expr.name)
	}
	e.state.runtimeErr(errExpectedObject, expr.name)
	return nil
}

func (e *execute) visitSetExpr(expr *setExpr) R {
	obj, ok := expr.object.accept(e).(grotskyInstance)
	if !ok {
		e.state.runtimeErr(errExpectedObject, expr.name)
	}
	obj.set(e.state, expr.name, expr.value.accept(e))
	return nil
}

func (e *execute) visitSuperExpr(expr *superExpr) R {
	superclass := e.env.get(e.state, expr.keyword).(*grotskyClass)
	// assert typeof(e.env.get(expr.keyword)) == (*grotskyClass)
	this := &token{
		token:  tkThis,
		lexeme: "this",
		line:   expr.keyword.line,
	}
	object := e.env.get(e.state, this).(*grotskyObject)
	// assert typeof(e.env.get(this)) == (*grotskyObject)
	method := superclass.findMethod(expr.method.lexeme)
	if method == nil {
		e.state.runtimeErr(errMethodNotFound, expr.method)
	}
	return method.bind(object)
}

func (e *execute) visitGroupingExpr(expr *groupingExpr) R {
	return expr.expression.accept(e)
}

func (e *execute) visitLiteralExpr(expr *literalExpr) R {
	return expr.value
}

func (e *execute) visitLogicalExpr(expr *logicalExpr) R {
	left := e.truthy(expr.left.accept(e))

	if expr.operator.token == tkOr {
		if left {
			return grotskyBool(true)
		}
		right := e.truthy(expr.right.accept(e))
		return grotskyBool(left || right)
	}

	// expr.operator.token = AND
	if !left {
		return grotskyBool(false)
	}
	right := e.truthy(expr.right.accept(e))
	return grotskyBool(left && right)
}

func (e *execute) visitThisExpr(expr *thisExpr) R {
	return e.env.get(e.state, expr.keyword)
}

func (e *execute) visitUnaryExpr(expr *unaryExpr) R {
	var err error
	value := expr.right.accept(e)
	switch expr.operator.token {
	case tkNot:
		return grotskyBool(!e.truthy(value))
	case tkMinus:
		value, err = e.operateUnary(opNeg, value)
	}
	if err != nil {
		e.state.runtimeErr(err, expr.operator)
	}
	return value
}

func (e *execute) truthy(value interface{}) bool {
	if value == nil {
		return false
	}
	valueStr, isStr := value.(grotskyString)
	if isStr {
		return valueStr != ""
	}
	valueNum, isNum := value.(grotskyNumber)
	if isNum {
		return valueNum != 0
	}
	valueBool, isBool := value.(grotskyBool)
	if isBool {
		return bool(valueBool)
	}
	return true
}

func (e *execute) visitVariableExpr(expr *variableExpr) R {
	return e.env.get(e.state, expr.name)
}

func (e *execute) visitFunctionExpr(expr *functionExpr) R {
	return &grotskyFunction{
		declaration: &fnStmt{body: expr.body, params: expr.params},
		closure:     e.env,
	}
}
