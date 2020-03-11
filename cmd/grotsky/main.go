package main

import (
	"fmt"
	"grotsky/internal"
	"io/ioutil"
	"log"
	"os"
)

func main() {
	argsWithoutProg := os.Args[1:]

	if len(argsWithoutProg) != 1 {
		fmt.Println("Usage: grotsky /path/to/source.g")
		return
	}

	file, err := os.Open(argsWithoutProg[0])
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	b, err := ioutil.ReadAll(file)
	if err != nil {
		log.Fatal(err)
	}

	source := string(b)

	state := internal.NewInterpreterState()

	l := internal.NewLexer(state)
	l.Scan()

	if !state.Valid() {
		state.PrintErrors()
		return
	}

	p := internal.NewParser(state)

}
