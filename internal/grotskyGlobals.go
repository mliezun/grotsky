package internal

import (
	"fmt"
	"net/http"
	"sync"
)

var gil sync.Mutex

type nativeFn struct {
	callFn func(arguments []interface{}) (interface{}, error)
}

func (n *nativeFn) call(arguments []interface{}) (interface{}, error) {
	return n.callFn(arguments)
}

type nativeObj struct {
	getFn         func(tk *token) interface{}
	setFn         func(name *token, value interface{})
	getOperatorFn func(op operator) (operatorApply, error)
}

func (n *nativeObj) get(tk *token) interface{} {
	if n.getFn == nil {
		state.runtimeErr(errUndefinedProp, tk)
	}
	return n.getFn(tk)
}

func (n *nativeObj) set(name *token, value interface{}) {
	if n.setFn == nil {
		state.runtimeErr(errReadOnly, name)
	}
	n.setFn(name, value)
}

func (n *nativeObj) getOperator(op operator) (operatorApply, error) {
	if n.getOperatorFn == nil {
		return nil, errUndefinedOp
	}
	return n.getOperatorFn(op)
}

func defineGlobals(e *env) {
	defineIo(e)
}

func defineIo(e *env) {
	var println nativeFn
	println.callFn = func(arguments []interface{}) (interface{}, error) {
		fmt.Println(arguments...)
		return nil, nil
	}

	e.define("io", &nativeObj{
		getFn: func(tk *token) interface{} {
			switch tk.lexeme {
			case "println":
				return &println
			default:
				state.runtimeErr(errUndefinedProp, tk)
			}
			return nil
		},
	})

	var handler nativeFn
	handler.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 2 {
			return nil, errInvalidNumberArguments
		}
		pattern, isString := arguments[0].(grotskyString)
		if !isString {
			return nil, errExpectedString
		}
		handle, isFunction := arguments[1].(grotskyCallable)
		if !isFunction {
			return nil, errExpectedFunction
		}
		http.HandleFunc(string(pattern), func(w http.ResponseWriter, req *http.Request) {
			gil.Lock()
			defer gil.Unlock()
			result, err := handle.call(nil)
			if err != nil {
				// TODO: handle error
			}
			resultDict, ok := result.(map[interface{}]interface{})
			if !ok {
				// TODO: handle error
			}
			// TODO: handle incorrect dict
			var body string
			for k, v := range resultDict {
				if fmt.Sprintf("%v", k) == "body" {
					body = string(v.(grotskyString))
				}
			}
			w.Write([]byte(body))
		})
		return nil, nil
	}

	var listen nativeFn
	listen.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		if addr, ok := arguments[0].(grotskyString); ok {
			http.ListenAndServe(string(addr), nil)
			return nil, nil
		}
		return nil, errExpectedString
	}

	e.define("http", &nativeObj{
		getFn: func(tk *token) interface{} {
			switch tk.lexeme {
			case "handler":
				return &handler
			case "listen":
				return &listen
			default:
				state.runtimeErr(errUndefinedProp, tk)
			}
			return nil
		},
	})
}
