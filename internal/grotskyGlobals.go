package internal

import (
	"errors"
	"io/ioutil"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"strings"
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
	defineType(e)
	defineImport(state, e)
	defineEnv(e)
	defineNet(e)
	defineStrings(e)
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

		// exec.mx.Unlock()
		// defer exec.mx.Lock()

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
		// exec.mx.Unlock()
		// defer exec.mx.Lock()
		return p.Println(arguments...)
	}

	e.define("io", &nativeObj{
		methods: map[string]*nativeFn{
			"println": &println,
		},
	})
}

func defineNet(e *env) {
	var listenTcp nativeFn
	listenTcp.callFn = func(arguments []interface{}) (interface{}, error) {
		// exec.mx.Unlock()
		// defer exec.mx.Lock()
		if len(arguments) != 1 {
			return nil, errInvalidNumberArguments
		}
		address, ok := arguments[0].(grotskyString)
		if !ok {
			return nil, errExpectedString
		}
		ln, err := net.Listen("tcp", string(address))
		if err != nil {
			return nil, err
		}
		listenObj := &nativeObj{
			methods: map[string]*nativeFn{
				"address": {
					callFn: func(arguments []interface{}) (interface{}, error) {
						return grotskyString(ln.Addr().String()), nil
					},
				},
				"close": {
					callFn: func(arguments []interface{}) (interface{}, error) {
						ln.Close()
						return nil, nil
					},
				},
				"accept": {
					callFn: func(arguments []interface{}) (interface{}, error) {
						conn, err := ln.Accept()
						if err != nil {
							return nil, err
						}
						return &nativeObj{
							methods: map[string]*nativeFn{
								"address": {
									callFn: func(arguments []interface{}) (interface{}, error) {
										return grotskyString(conn.RemoteAddr().String()), nil
									},
								},
								"read": {
									callFn: func(arguments []interface{}) (interface{}, error) {
										buffer := make([]byte, 4096)
										size, err := conn.Read(buffer)
										if err != nil {
											return nil, err
										}
										return grotskyString(buffer[:size]), nil
									},
								},
								"write": {
									callFn: func(arguments []interface{}) (interface{}, error) {
										if len(arguments) != 1 {
											return nil, errInvalidNumberArguments
										}
										content, ok := arguments[0].(grotskyString)
										if !ok {
											return nil, errExpectedString
										}
										n, err := conn.Write([]byte(content))
										if err != nil {
											return nil, err
										}
										return grotskyNumber(n), nil
									},
								},
								"close": {
									callFn: func(arguments []interface{}) (interface{}, error) {
										conn.Close()
										return nil, nil
									},
								},
							},
						}, nil
					},
				},
			},
		}
		return listenObj, nil
	}

	e.define("net", &nativeObj{
		methods: map[string]*nativeFn{
			"listenTcp": &listenTcp,
		},
	})
}

func defineStrings(e *env) {
	toLower := nativeFn{
		callFn: func(arguments []interface{}) (interface{}, error) {
			if len(arguments) != 1 {
				return nil, errInvalidNumberArguments
			}
			str, _ := arguments[0].(grotskyString)
			return grotskyString(strings.ToLower(string(str))), nil
		},
	}
	toUpper := nativeFn{
		callFn: func(arguments []interface{}) (interface{}, error) {
			if len(arguments) != 1 {
				return nil, errInvalidNumberArguments
			}
			str, _ := arguments[0].(grotskyString)
			return grotskyString(strings.ToUpper(string(str))), nil
		},
	}
	ord := nativeFn{
		callFn: func(arguments []interface{}) (interface{}, error) {
			if len(arguments) != 1 {
				return nil, errInvalidNumberArguments
			}
			str, _ := arguments[0].(grotskyString)
			runes := []rune(str)
			return grotskyNumber(runes[0]), nil
		},
	}
	chr := nativeFn{
		callFn: func(arguments []interface{}) (interface{}, error) {
			if len(arguments) != 1 {
				return nil, errInvalidNumberArguments
			}
			x, _ := arguments[0].(grotskyNumber)
			return grotskyString(rune(x)), nil
		},
	}
	asNumber := nativeFn{
		callFn: func(arguments []interface{}) (interface{}, error) {
			if len(arguments) != 1 {
				return nil, errInvalidNumberArguments
			}
			str, _ := arguments[0].(grotskyString)
			num, err := strconv.ParseFloat(string(str), 64)
			if err != nil {
				return nil, nil
			}
			return grotskyNumber(num), nil
		},
	}

	e.define("strings", &nativeObj{
		methods: map[string]*nativeFn{
			"toLower":  &toLower,
			"toUpper":  &toUpper,
			"ord":      &ord,
			"chr":      &chr,
			"asNumber": &asNumber,
		},
	})
}
