package internal

import "fmt"

type grotskyCallable interface {
	arity() int
	call(exec *exec, arguments []interface{}) interface{}
}

type grotskyFunction struct {
	declaration   *fnStmt
	closure       *env
	isInitializer bool
}

type nativeFn struct {
	arityValue int
	callFn     func(exec *exec, arguments []interface{}) interface{}
}

func (n *nativeFn) arity() int {
	return n.arityValue
}

func (n *nativeFn) call(exec *exec, arguments []interface{}) interface{} {
	return n.callFn(exec, arguments)
}

func (f *grotskyFunction) arity() int {
	return len(f.declaration.params)
}

func (f *grotskyFunction) call(exec *exec, arguments []interface{}) (result interface{}) {
	env := newEnv(exec.state, f.closure)
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
			return exec.executeOne(exprSt, env)
		}
	}

	exec.executeBlock(f.declaration.body, env)

	return nil
}

func (f *grotskyFunction) bind(object *grotskyObject) *grotskyFunction {
	environment := newEnv(f.closure.state, f.closure)
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
