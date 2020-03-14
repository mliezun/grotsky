package main

import (
	"fmt"
	"os"
	"strings"
)

//go:generate ./ast.sh

func main() {
	var out string
	switch os.Args[1] {
	case "Stmt":
		out = generateAst("Stmt", []string{
			"Expr: expression Expr",
		})
	case "Expr":
		out = generateAst("Expr", []string{
			"Assign: name *Token, value Expr",
			"Binary: left Expr, operator *Token, right Expr",
			"Call: callee Expr, paren *Token, arguments []Expr",
			"Get: object Expr, name *Token",
			"Set: object Expr, name *Token, value Expr",
			"Super: keyword *Token, method *Token",
			"Grouping: expression Expr",
			"Literal: value interface{}",
			"Logical: left Expr, operator *Token, right Expr",
			"This: keyword *Token",
			"Unary: operator *Token, right Expr",
			"Variable: name *Token",
			"Function: params []*Token, body []Stmt",
		})
	}
	fmt.Println(out)
}

func generateAst(baseName string, types []string) string {
	out := "package internal\n\n"

	// Start base interface
	out += "type " + baseName + " interface {\n"
	out += "\taccept(" + baseName + "Visitor) R\n"
	out += "}\n\n"
	// End base interface

	// Start Visitor interface
	out += fmt.Sprintf("type %sVisitor interface {\n", baseName)
	for _, t := range types {
		typeDef := strings.Split(t, ":")
		structName := strings.TrimSpace(typeDef[0])
		out += "\tvisit" + structName + baseName + "(" + strings.ToLower(baseName) + " " + baseName + ") R\n"
	}
	out += "}\n\n"
	// End Visitor interface

	// Start  structs
	for _, t := range types {
		typeDef := strings.Split(t, ":")
		structName := strings.TrimSpace(typeDef[0])
		structFields := strings.TrimSpace(typeDef[1])
		out += generateType(baseName, structName, structFields)
	}
	// End structs

	return out
}

func generateType(baseName, name, fields string) string {
	// Start Structure Definition
	out := "type " + name + baseName + " struct {\n"
	fieldArray := strings.Split(fields, ",")
	for _, field := range fieldArray {
		out += "\t" + strings.TrimSpace(field) + "\n"
	}
	out += "}\n\n"
	// End Structure Definition

	// Start Method Definition
	out += "func (s *" + name + baseName + ") accept(visitor " + baseName + "Visitor) R {\n"
	out += "\treturn visitor.visit" + name + baseName + "(s)\n"
	out += "}\n\n"
	// End Method Definition

	return out
}
