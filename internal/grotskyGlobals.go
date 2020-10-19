package internal

import (
	"io/ioutil"
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

func (n *nativeFn) String() string {
	return "<fn native>"
}

type nativeObj struct {
	properties    map[string]interface{}
	methods       map[string]*nativeFn
	setFn         func(name *token, value interface{})
	getOperatorFn func(op operator) (operatorApply, error)
}

func (n *nativeObj) get(tk *token) interface{} {
	if prop, ok := n.properties[tk.lexeme]; ok {
		return prop
	}
	if method, ok := n.methods[tk.lexeme]; ok {
		return method
	}
	state.runtimeErr(errUndefinedProp, tk)
	return nil
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

func (n *nativeObj) String() string {
	return "<instance native>"
}

func defineGlobals(e *env, p IPrinter) {
	defineIo(e, p)
	defineHTTP(e)
}

func defineIo(e *env, p IPrinter) {
	var println nativeFn
	println.callFn = func(arguments []interface{}) (interface{}, error) {
		return p.Println(arguments...)
	}

	e.define("io", &nativeObj{
		methods: map[string]*nativeFn{
			"println": &println,
		},
	})
}

func defineHTTP(e *env) {
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
			responseWriter := &nativeObj{
				methods: map[string]*nativeFn{
					"write": {
						callFn: func(arguments []interface{}) (interface{}, error) {
							if len(arguments) != 2 {
								// TODO: handle error
							}
							code, ok := arguments[0].(grotskyNumber)
							if !ok {
								// TODO: handle error
							}
							w.WriteHeader(int(code))
							if _, err := w.Write([]byte(arguments[1].(grotskyString))); err != nil {
								// TODO: handle error
							}
							return nil, nil
						},
					},
				},
			}

			requestReader := &nativeObj{
				properties: map[string]interface{}{
					"method":    grotskyString(req.Method),
					"address":   grotskyString(req.RemoteAddr),
					"userAgent": grotskyString(req.UserAgent()),
				},
				methods: map[string]*nativeFn{
					"body": {
						callFn: func(arguments []interface{}) (interface{}, error) {
							if len(arguments) != 0 {
								// TODO: handle error
							}
							rqBody, err := ioutil.ReadAll(req.Body)
							if err != nil {
								// TODO: handle error
							}
							return grotskyString(rqBody), nil
						},
					},
				},
			}

			arguments = []interface{}{
				requestReader,
				responseWriter,
			}

			gil.Lock()
			defer gil.Unlock()
			_, err := handle.call(arguments)
			if err != nil {
				// TODO: handle error
			}
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
		methods: map[string]*nativeFn{
			"handler": &handler,
			"listen":  &listen,
		},
	})
}
