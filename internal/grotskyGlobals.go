package internal

import (
	"errors"
	"io/ioutil"
	"net/http"
	"os"
	"path/filepath"
)

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

func (n *nativeObj) get(state *interpreterState, tk *token) interface{} {
	if prop, ok := n.properties[tk.lexeme]; ok {
		return prop
	}
	if method, ok := n.methods[tk.lexeme]; ok {
		return method
	}
	state.runtimeErr(errUndefinedProp, tk)
	return nil
}

func (n *nativeObj) set(state *interpreterState, name *token, value interface{}) {
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

func defineGlobals(state *interpreterState, e *env, p IPrinter) {
	defineIo(e, p)
	defineHTTP(state, e)
	defineType(e)
	defineImport(state, e)
	defineEnv(e)
}

func defineType(e *env) {
	var typeFn nativeFn
	typeFn.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		switch arguments[0].(type) {
		case grotskyBool:
			return grotskyString("bool"), nil
		case *grotskyClass:
			return grotskyString("class"), nil
		case grotskyDict:
			return grotskyString("dict"), nil
		case *grotskyFunction:
			return grotskyString("function"), nil
		case grotskyList:
			return grotskyString("list"), nil
		case grotskyNumber:
			return grotskyString("number"), nil
		case *grotskyObject:
			return grotskyString("object"), nil
		case grotskyString:
			return grotskyString("string"), nil
		}
		// nil type
		return grotskyString("nil"), nil
	}

	e.define("type", &typeFn)
}

func defineImport(state *interpreterState, e *env) {
	var importFn nativeFn
	importFn.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		pathGr, ok := arguments[0].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		modulePath := string(pathGr)

		if !filepath.IsAbs(modulePath) {
			basePath := filepath.Dir(state.absPath)
			modulePath = filepath.Join(basePath, modulePath)
		}

		exec.mx.Unlock()
		defer exec.mx.Lock()

		file, err := os.Open(modulePath)
		if err != nil {
			return nil, err
		}
		defer file.Close()

		b, err := ioutil.ReadAll(file)
		if err != nil {
			return nil, err
		}

		moduleEnv, ok := importModule(state, modulePath, string(b))
		if !ok || moduleEnv == nil {
			return nil, errors.New("import module error")
		}

		// moduleEnv has no enclosing
		return &nativeObj{properties: moduleEnv.values}, nil
	}

	e.define("import", &importFn)
}

func defineEnv(e *env) {
	var getFn nativeFn
	getFn.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		envVar, ok := arguments[0].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		return grotskyString(os.Getenv(string(envVar))), nil
	}

	var setFn nativeFn
	setFn.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 2 {
			return nil, errInvalidNumberArguments
		}
		envVar, ok := arguments[0].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		val, ok := arguments[1].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		return nil, os.Setenv(string(envVar), string(val))
	}

	e.define("env", &nativeObj{
		methods: map[string]*nativeFn{
			"get": &getFn,
			"set": &setFn,
		},
	})
}

func defineIo(e *env, p IPrinter) {
	var println nativeFn
	println.callFn = func(arguments []interface{}) (interface{}, error) {
		exec.mx.Unlock()
		defer exec.mx.Lock()
		return p.Println(arguments...)
	}

	var readfile nativeFn
	readfile.callFn = func(arguments []interface{}) (interface{}, error) {
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		pathGr, ok := arguments[0].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		exec.mx.Unlock()
		defer exec.mx.Lock()
		file, err := os.Open(string(pathGr))
		if err != nil {
			return nil, err
		}
		defer file.Close()

		b, err := ioutil.ReadAll(file)
		if err != nil {
			return nil, err
		}

		return grotskyString(b), nil
	}

	e.define("io", &nativeObj{
		methods: map[string]*nativeFn{
			"println":  &println,
			"readfile": &readfile,
		},
	})
}

func defineHTTP(state *interpreterState, e *env) {
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
								return nil, errInvalidNumberArguments
							}
							code, ok := arguments[0].(grotskyNumber)
							if !ok {
								return nil, errExpectedNumber
							}
							exec.mx.Unlock()
							defer exec.mx.Lock()
							w.WriteHeader(int(code))
							body := arguments[1].(grotskyString)
							if _, err := w.Write([]byte(body)); err != nil {
								return nil, err
							}
							return nil, nil
						},
					},
				},
			}

			grotskyHeader := make(grotskyDict)
			for header, vals := range req.Header {
				grotskyVals := make(grotskyList, len(vals))
				for i := range vals {
					grotskyVals[i] = grotskyString(vals[i])
				}
				grotskyHeader[grotskyString(header)] = grotskyVals
			}

			requestReader := &nativeObj{
				properties: map[string]interface{}{
					"method":    grotskyString(req.Method),
					"address":   grotskyString(req.RemoteAddr),
					"userAgent": grotskyString(req.UserAgent()),
					"host":      grotskyString(req.Host),
					"uri":       grotskyString(req.RequestURI),
					"referer":   grotskyString(req.Referer()),
					"headers":   grotskyHeader,
				},
				methods: map[string]*nativeFn{
					"body": {
						callFn: func(arguments []interface{}) (interface{}, error) {
							if len(arguments) != 0 {
								return nil, errInvalidNumberArguments
							}
							exec.mx.Unlock()
							defer exec.mx.Lock()
							rqBody, err := ioutil.ReadAll(req.Body)
							if err != nil {
								return nil, err
							}
							return grotskyString(rqBody), nil
						},
					},
				},
			}

			exec.mx.Lock()
			defer exec.mx.Unlock()
			_, err := handle.call([]interface{}{
				requestReader,
				responseWriter,
			})
			if err != nil {
				w.WriteHeader(http.StatusInternalServerError)
				state.logger.Println(err)
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
			exec.mx.Unlock()
			defer exec.mx.Lock()
			return nil, http.ListenAndServe(string(addr), nil)
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
