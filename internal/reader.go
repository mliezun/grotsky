package internal

import "fmt"

//R generic type
type R interface{}

func printObj(a interface{}) string {
	if r, ok := a.(Representable); ok {
		return r.Repr()
	}
	return fmt.Sprintf("%v", a)
}
