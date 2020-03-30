package internal

import "fmt"

type callable interface {
	arity() int
	call(exec *exec, arguments []interface{}) interface{}
}

type function struct {
	declaration   *fnStmt
	closure       *env
	isInitializer bool
}

type nativeFn struct {
	arityFn func() int
	callFn  func(exec *exec, arguments []interface{}) interface{}
}

func (n *nativeFn) arity() int {
	return n.arityFn()
}

func (n *nativeFn) call(exec *exec, arguments []interface{}) interface{} {
	return n.callFn(exec, arguments)
}

func (f *function) arity() int {
	return len(f.declaration.params)
}

func (f *function) call(exec *exec, arguments []interface{}) (result interface{}) {
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

	exec.executeBlock(f.declaration.body, env)

	return nil
}

func (f *function) String() string {
	return fmt.Sprintf("<fn %s>", f.declaration.name.lexeme)
}
