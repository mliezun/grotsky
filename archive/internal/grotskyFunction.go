package internal

import (
	"fmt"
)

type grotskyCallable interface {
	call(arguments []interface{}) (interface{}, error)
	String() string
}

type grotskyFunction struct {
	declaration   *fnStmt[R]
	closure       *env
	isInitializer bool
}

func (f *grotskyFunction) call(arguments []interface{}) (result interface{}, err error) {
	env := newEnv(f.closure)

	if len(arguments) != len(f.declaration.params) {
		return nil, errInvalidNumberArguments
	}

	for i := range f.declaration.params {
		env.define(f.declaration.params[i].lexeme, arguments[i])
	}

	if len(f.declaration.body) == 1 {
		if exprSt, ok := f.declaration.body[0].(*exprStmt[R]); ok {
			resultVal := exec.executeOne(exprSt, env)
			return resultVal, nil
		}
	}

	resultVal := exec.executeBlock(f.declaration.body, env)
	if returnVal, isReturn := resultVal.(*returnValue); isReturn {
		return returnVal.value, nil
	}

	return resultVal, nil
}

func (f *grotskyFunction) bind(object *grotskyObject) *grotskyFunction {
	environment := newEnv(f.closure)
	environment.define("this", object)
	return &grotskyFunction{
		declaration: f.declaration,
		closure:     environment,
	}
}

func (f *grotskyFunction) String() string {
	name := "anonymous"
	if f.declaration.name != nil {
		name = f.declaration.name.lexeme
	}
	return fmt.Sprintf("<fn %s>", name)
}
