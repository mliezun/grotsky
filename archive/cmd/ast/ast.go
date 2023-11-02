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
			"Expr: last *token, expression expr[T]",
			"TryCatch: tryBody stmt[T], name *token, catchBody stmt[T]",
			"ClassicFor: keyword *token, initializer stmt[T], condition expr[T], increment expr[T], body stmt[T]",
			"EnhancedFor: keyword *token, identifiers []*token, collection expr[T], body stmt[T]",
			"Let: name *token, initializer expr[T]",
			"Block: stmts []stmt[T]",
			"While: keyword *token, condition expr[T], body stmt[T]",
			"Return: keyword *token, value expr[T]",
			"Break: keyword *token",
			"Continue: keyword *token",
			"If: keyword *token, condition expr[T], thenBranch []stmt[T], elifs []*struct{condition expr[T]; thenBranch []stmt[T]}, elseBranch []stmt[T]",
			"Fn: name *token, params []*token, body []stmt[T]",
			"Class: name *token, superclass *variableExpr[T], methods []*fnStmt[T], staticMethods []*fnStmt[T]",
		})
	case "Expr":
		out = generateAst("Expr", []string{
			"List: elements []expr[T], brace *token",
			"Dictionary: elements []expr[T], curlyBrace *token",
			"Assign: name *token, value expr[T], access expr[T]",
			"Access: object expr[T], brace *token, first expr[T], firstColon *token, second expr[T], secondColon *token, third expr[T]",
			"Binary: left expr[T], operator *token, right expr[T]",
			"Call: callee expr[T], paren *token, arguments []expr[T]",
			"Get: object expr[T], name *token",
			"Set: object expr[T], name *token, value expr[T], access expr[T]",
			"Super: keyword *token, method *token",
			"Grouping: expression expr[T]",
			"Literal: value interface{}",
			"Logical: left expr[T], operator *token, right expr[T]",
			"This: keyword *token",
			"Unary: operator *token, right expr[T]",
			"Variable: name *token",
			"Function: params []*token, body []stmt[T]",
		})
	}
	fmt.Println(out)
}

func generateAst(baseName string, types []string) string {
	out := "package internal\n\n"

	// Start base interface
	out += "type " + strings.ToLower(baseName) + "[T any] interface {\n"
	out += "\taccept(" + strings.ToLower(baseName) + "Visitor[T]) T\n"
	out += "}\n\n"
	// End base interface

	// Start Visitor interface
	out += fmt.Sprintf("type %sVisitor[T any] interface {\n", strings.ToLower(baseName))
	for _, t := range types {
		typeDef := strings.Split(t, ":")
		name := strings.TrimSpace(typeDef[0])
		structType := strings.ToLower(string(name[0])) + name[1:] + baseName
		out += "\tvisit" + name + baseName + "(" + strings.ToLower(baseName) + " *" + structType + "[T]) T\n"
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
	structName := strings.ToLower(string(name[0])) + name[1:] + baseName
	out := "type " + structName + "[T any] struct {\n"
	fieldArray := strings.Split(fields, ",")
	for _, field := range fieldArray {
		out += "\t" + strings.TrimSpace(field) + "\n"
	}
	out += "}\n\n"
	// End Structure Definition

	// Start Method Definition
	out += "func (s *" + structName + "[T]) accept(visitor " + strings.ToLower(baseName) + "Visitor[T]) T {\n"
	out += "\treturn visitor.visit" + name + baseName + "(s)\n"
	out += "}\n\n"
	// End Method Definition

	return out
}
