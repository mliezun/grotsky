package internal

import "fmt"

//R generic type
type R interface{}

func printObj(a interface{}) string {
	return fmt.Sprintf("%v", a)
}
