package main

import (
	"fmt"
	"grotsky/internal"
	"io"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"
)

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
	argsWithoutProg := os.Args[1:]

	if len(argsWithoutProg) != 1 {
		fmt.Println("Usage: grotsky /path/to/source.g")
		return
	}

	absPath, err := filepath.Abs(argsWithoutProg[0])
	if err != nil {
		log.Fatal(err)
	}

	file, err := os.Open(absPath)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	b, err := ioutil.ReadAll(file)
	if err != nil {
		log.Fatal(err)
	}

	source := string(b)

	internal.RunSourceWithPrinter(absPath, source, stdPrinter{})
}
