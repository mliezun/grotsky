package internal

import "fmt"

type grotskyCallable interface {
	call(arguments []interface{}) (interface{}, error)
}

type grotskyFunction struct {
	declaration   *fnStmt
	closure       *env
	isInitializer bool
}

type nativeFn struct {
	callFn func(arguments []interface{}) (interface{}, error)
}

func (n *nativeFn) call(arguments []interface{}) (interface{}, error) {
	return n.callFn(arguments)
}

func (f *grotskyFunction) call(arguments []interface{}) (result interface{}, err error) {
	env := newEnv(f.closure)

	if len(arguments) != len(f.declaration.params) {
		return nil, errInvalidNumberArguments
	}

	for i := range f.declaration.params {
		env.define(f.declaration.params[i].lexeme, arguments[i])
	}

	defer func() {
		if r := recover(); r != nil {
			if returnVal, isReturn := r.(returnValue); isReturn {
				result = returnVal
			} else {
				panic(r)
			}
		}
	}()

	if len(f.declaration.body) == 1 {
		if exprSt, ok := f.declaration.body[0].(*exprStmt); ok {
			return exec.executeOne(exprSt, env), nil
		}
	}

	exec.executeBlock(f.declaration.body, env)

	return nil, nil
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
