package internal

type execute struct {
	globals *env
	env     *env
}

var exec = execute{}

func init() {
	exec.env = newEnv(nil)
	exec.globals = exec.env
}

func (e execute) interpret() {
	defer func() {
		if state.runtimeError != nil {
			recover()
		}
	}()
	for _, s := range state.stmts {
		s.accept(e)
	}
}

func (e execute) visitExprStmt(stmt *exprStmt) R {
	return stmt.expression.accept(e)
}

func (e execute) visitClassicForStmt(stmt *classicForStmt) R {
	for stmt.initializer.accept(e); e.truthy(stmt.condition.accept(e)); stmt.increment.accept(e) {
		if returnVal, isReturn := stmt.body.accept(e).(returnValue); isReturn {
			return returnVal
		}
	}
	return nil
}

func (e execute) visitEnhancedForStmt(stmt *enhancedForStmt) R {
	collection := stmt.collection.accept(e)
	environment := newEnv(e.env)

	identifiersCount := len(stmt.identifiers)

	if array, ok := collection.(grotskyList); ok {
		for _, el := range array {
			if identifiersCount == 1 {
				environment.define(stmt.identifiers[0].lexeme, el)
			} else if array2, ok := el.(grotskyList); ok {
				if len(array2) != identifiersCount {
					state.runtimeErr(errWrongNumberOfValues, stmt.keyword)
				}
				for i, id := range stmt.identifiers {
					environment.define(id.lexeme, array2[i])
				}
			} else {
				state.runtimeErr(errCannotUnpack, stmt.keyword)
			}
			if returnVal, isReturn := e.executeOne(stmt.body, environment).(returnValue); isReturn {
				return returnVal
			}
		}
	} else if dict, ok := collection.(grotskyDict); ok {
		if identifiersCount > 2 {
			state.runtimeErr(errExpectedIdentifiersDict, stmt.keyword)
		}
		for key, value := range dict {
			if identifiersCount == 1 {
				environment.define(stmt.identifiers[0].lexeme, key)
			} else {
				environment.define(stmt.identifiers[0].lexeme, key)
				environment.define(stmt.identifiers[1].lexeme, value)
			}
			if returnVal, isReturn := e.executeOne(stmt.body, environment).(returnValue); isReturn {
				return returnVal
			}
		}
	} else {
		state.runtimeErr(errExpectedCollection, stmt.keyword)
	}

	return nil
}

func (e execute) visitLetStmt(stmt *letStmt) R {
	var val interface{}
	if stmt.initializer != nil {
		val = stmt.initializer.accept(e)
	}
	e.env.define(stmt.name.lexeme, val)
	return nil
}

func (e execute) visitBlockStmt(stmt *blockStmt) R {
	return e.executeBlock(stmt.stmts, newEnv(e.env))
}

func (e execute) executeOne(st stmt, env *env) R {
	previous := e.env
	defer func() {
		e.env = previous
	}()
	e.env = env
	return st.accept(e)
}

func (e execute) executeBlock(stmts []stmt, env *env) R {
	previous := e.env
	defer func() {
		e.env = previous
	}()
	e.env = env
	for _, s := range stmts {
		if returnVal, isReturn := s.accept(e).(returnValue); isReturn {
			return returnVal
		}
	}
	return nil
}

func (e execute) visitWhileStmt(stmt *whileStmt) R {
	for e.truthy(stmt.condition.accept(e)) {
		if returnVal, isReturn := stmt.body.accept(e).(returnValue); isReturn {
			return returnVal
		}
	}
	return nil
}

func (e execute) visitReturnStmt(stmt *returnStmt) R {
	if stmt.value != nil {
		return returnValue(stmt.value.accept(e))
	}
	return nil
}

func (e execute) visitIfStmt(stmt *ifStmt) R {
	if e.truthy(stmt.condition.accept(e)) {
		for _, st := range stmt.thenBranch {
			if returnVal, isReturn := st.accept(e).(returnValue); isReturn {
				return returnVal
			}
		}
		return nil
	}
	for _, elif := range stmt.elifs {
		if e.truthy(elif.condition.accept(e)) {
			for _, st := range elif.thenBranch {
				if returnVal, isReturn := st.accept(e).(returnValue); isReturn {
					return returnVal
				}
			}
			return nil
		}
	}
	if stmt.elseBranch != nil {
		for _, st := range stmt.elseBranch {
			if returnVal, isReturn := st.accept(e).(returnValue); isReturn {
				return returnVal
			}
		}
	}
	return nil
}

func (e execute) visitFnStmt(stmt *fnStmt) R {
	e.env.define(stmt.name.lexeme, &grotskyFunction{
		declaration:   stmt,
		closure:       e.env,
		isInitializer: false,
	})
	return nil
}

func (e execute) visitClassStmt(stmt *classStmt) R {
	class := &grotskyClass{
		name: stmt.name.lexeme,
	}

	if stmt.superclass != nil {
		superclass, ok := stmt.superclass.accept(e).(*grotskyClass)
		if !ok {
			state.runtimeErr(errExpectedClass, stmt.name)
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

func (e execute) visitListExpr(expr *listExpr) R {
	list := make(grotskyList, len(expr.elements))
	for i, el := range expr.elements {
		list[i] = el.accept(e)
	}
	return list
}

func (e execute) visitDictionaryExpr(expr *dictionaryExpr) R {
	dict := make(grotskyDict)
	for i := 0; i < len(expr.elements)/2; i++ {
		dict[expr.elements[i*2].accept(e)] = expr.elements[i*2+1].accept(e)
	}
	return dict
}

func (e execute) visitAssignExpr(expr *assignExpr) R {
	val := expr.value.accept(e)
	e.env.assign(expr.name, val)
	return val
}

func (e execute) visitAccessExpr(expr *accessExpr) R {
	object := expr.object.accept(e)
	list, isList := object.(grotskyList)
	if isList {
		return e.sliceList(list, expr)
	}
	dict, isDict := object.(grotskyDict)
	if isDict {
		if expr.first == nil {
			state.runtimeErr(errExpectedKey, expr.brace)
		}
		return dict[expr.first.accept(e)]
	}
	state.runtimeErr(errInvalidAccess, expr.brace)
	return nil
}

func (e execute) sliceList(list grotskyList, accessExpr *accessExpr) interface{} {
	first, second, third := e.exprToInt(accessExpr.first, accessExpr.brace),
		e.exprToInt(accessExpr.second, accessExpr.brace),
		e.exprToInt(accessExpr.third, accessExpr.brace)

	if first != nil {
		if accessExpr.firstColon != nil {
			if second != nil {
				// [a:b:c]
				if accessExpr.secondColon != nil {
					if third == nil {
						state.runtimeErr(errExpectedStep, accessExpr.secondColon)
					}
					return e.stepList(list[*first:*second], *third)
				}
				// [a:b]
				return list[*first:*second]
			}

			// [a::c]
			if accessExpr.secondColon != nil {
				if third == nil {
					state.runtimeErr(errExpectedStep, accessExpr.secondColon)
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
				state.runtimeErr(errExpectedStep, accessExpr.secondColon)
			}
			return e.stepList(list[:*second], *third)
		}
		// [:b]
		return list[:*second]
	}

	if third == nil {
		state.runtimeErr(errExpectedStep, accessExpr.secondColon)
	}
	// [::c]
	return e.stepList(list, *third)
}

func (e execute) exprToInt(expr expr, token *token) *int64 {
	if expr == nil {
		return nil
	}
	valueF, ok := expr.accept(e).(float64)
	if !ok {
		state.runtimeErr(errOnlyNumbers, token)
	}
	valueI := int64(valueF)
	return &valueI
}

func (e execute) stepList(list grotskyList, step int64) grotskyList {
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

func (e execute) visitBinaryExpr(expr *binaryExpr) R {
	var value interface{}
	var err error
	left := expr.left.accept(e)
	right := expr.right.accept(e)
	switch expr.operator.token {
	case EQUAL_EQUAL:
		value, err = e.operateBinary(opEq, left, right)
	case BANG_EQUAL:
		value, err = e.operateBinary(opNeq, left, right)
	case GREATER:
		value, err = e.operateBinary(opGt, left, right)
	case GREATER_EQUAL:
		value, err = e.operateBinary(opGte, left, right)
	case LESS:
		value, err = e.operateBinary(opLt, left, right)
	case LESS_EQUAL:
		value, err = e.operateBinary(opLte, left, right)
	case PLUS:
		value, err = e.operateBinary(opAdd, left, right)
	case MINUS:
		value, err = e.operateBinary(opSub, left, right)
	case SLASH:
		value, err = e.operateBinary(opDiv, left, right)
	case STAR:
		value, err = e.operateBinary(opMul, left, right)
	case POWER:
		value, err = e.operateBinary(opPow, left, right)
	default:
		state.runtimeErr(errUndefinedOp, expr.operator)
	}
	if err != nil {
		state.runtimeErr(err, expr.operator)
	}
	return value
}

func (e execute) operateUnary(op operator, left interface{}) (interface{}, error) {
	leftVal := left.(grotskyInstance)
	apply, err := leftVal.getOperator(op)
	if err != nil {
		return nil, err
	}
	return apply()
}

func (e execute) operateBinary(op operator, left, right interface{}) (interface{}, error) {
	leftVal := left.(grotskyInstance)
	apply, err := leftVal.getOperator(op)
	if err != nil {
		return nil, err
	}
	return apply(right)
}

func (e execute) visitCallExpr(expr *callExpr) R {
	callee := expr.callee.accept(e)
	arguments := make([]interface{}, len(expr.arguments))
	for i := range expr.arguments {
		arguments[i] = expr.arguments[i].accept(e)
	}

	fn, isFn := callee.(grotskyCallable)
	if !isFn {
		state.runtimeErr(errOnlyFunction, expr.paren)
	}

	result, err := fn.call(arguments)
	if err != nil {
		state.runtimeErr(err, expr.paren)
	}

	return result
}

func (e execute) visitGetExpr(expr *getExpr) R {
	object := expr.object.accept(e)
	if obj, ok := object.(grotskyInstance); ok {
		return obj.get(expr.name)
	}
	state.runtimeErr(errExpectedObject, expr.name)
	return nil
}

func (e execute) visitSetExpr(expr *setExpr) R {
	obj, ok := expr.object.accept(e).(grotskyInstance)
	if !ok {
		state.runtimeErr(errExpectedObject, expr.name)
	}
	obj.set(expr.name, expr.value.accept(e))
	return nil
}

func (e execute) visitSuperExpr(expr *superExpr) R {
	superclass, ok := e.env.get(expr.keyword).(*grotskyClass)
	if !ok {
		state.runtimeErr(errExpectedSuperclass, expr.keyword)
	}
	this := &token{
		token:  THIS,
		lexeme: "this",
		line:   expr.keyword.line,
	}
	object, ok := e.env.get(this).(*grotskyObject)
	if !ok {
		state.runtimeErr(errExpectedObject, expr.keyword)
	}
	method := superclass.findMethod(expr.method.lexeme)
	if method == nil {
		state.runtimeErr(errMethodNotFound, expr.method)
	}
	return method.bind(object)
}

func (e execute) visitGroupingExpr(expr *groupingExpr) R {
	return expr.expression.accept(e)
}

func (e execute) visitLiteralExpr(expr *literalExpr) R {
	return expr.value
}

func (e execute) visitLogicalExpr(expr *logicalExpr) R {
	left := e.truthy(expr.left.accept(e))

	if expr.operator.token == OR {
		if left {
			return grotskyBool(true)
		}
		right := e.truthy(expr.right.accept(e))
		return grotskyBool(left || right)
	}

	if expr.operator.token == AND {
		if !left {
			return grotskyBool(false)
		}
		right := e.truthy(expr.right.accept(e))
		return grotskyBool(left && right)
	}

	state.runtimeErr(errUndefinedOp, expr.operator)

	return nil
}

func (e execute) visitThisExpr(expr *thisExpr) R {
	return e.env.get(expr.keyword)
}

func (e execute) visitUnaryExpr(expr *unaryExpr) R {
	var err error
	value := expr.right.accept(e)
	switch expr.operator.token {
	case NOT:
		return grotskyBool(!e.truthy(value))
	case MINUS:
		value, err = e.operateUnary(opNeg, value)
	default:
		state.runtimeErr(errUndefinedOp, expr.operator)
	}
	if err != nil {
		state.runtimeErr(err, expr.operator)
	}
	return value
}

func (e execute) truthy(value interface{}) bool {
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

func (e execute) visitVariableExpr(expr *variableExpr) R {
	return e.env.get(expr.name)
}

func (e execute) visitFunctionExpr(expr *functionExpr) R {
	return &grotskyFunction{
		declaration: &fnStmt{body: expr.body, params: expr.params},
		closure:     e.env,
	}
}
