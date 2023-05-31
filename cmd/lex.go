package main

import (
	"fmt"
	"io"
	"time"

	"github.com/mliezun/grotsky/internal"
)

var source string = `
let a = 1
while a < 100000000 {
    a = a + 1
}
`

type stdPrinter struct{}

func (s stdPrinter) Println(a ...interface{}) (n int, err error) {
	return fmt.Println(a...)
}

func (s stdPrinter) Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error) {
	return fmt.Fprintf(w, format, a...)
}

func (s stdPrinter) Fprintln(w io.Writer, a ...interface{}) (n int, err error) {
	return fmt.Fprintln(w, a...)
}

func main() {
	start := time.Now()
	internal.RunSourceWithPrinter("", source, stdPrinter{})
	fmt.Println("Time elapsed is:", time.Since(start))
}
