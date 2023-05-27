package main

import "github.com/mliezun/grotsky/internal"

var source string = `
let list = [[1], 2, 3] # guaska
io.println(list[0][0])
`

func main() {
	internal.LexSource(source)
}
