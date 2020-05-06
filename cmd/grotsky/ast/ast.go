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
			"Expr: last *token, expression expr",
			"ClassicFor: keyword *token, initializer stmt, condition expr, increment expr, body stmt",
			"EnhancedFor: keyword *token, identifiers []*token, collection expr, body stmt",
			"Let: name *token, initializer expr",
			"Block: stmts []stmt",
			"While: keyword *token, condition expr, body stmt",
			"Return: keyword *token, value expr",
			"If: keyword *token, condition expr, thenBranch []stmt, elifs []*struct{condition expr; thenBranch []stmt}, elseBranch []stmt",
			"Fn: name *token, params []*token, body []stmt",
			"Class: name *token, superclass *variableExpr, methods []*fnStmt, staticMethods []*fnStmt",
		})
	case "Expr":
		out = generateAst("Expr", []string{
			"List: elements []expr, brace *token",
			"Dictionary: elements []expr, curlyBrace *token",
			"Assign: name *token, value expr",
			"Access: object expr, brace *token, first expr, firstColon *token, second expr, secondColon *token, third expr",
			"Binary: left expr, operator *token, right expr",
			"Call: callee expr, paren *token, arguments []expr",
			"Get: object expr, name *token",
			"Set: object expr, name *token, value expr",
			"Super: keyword *token, method *token",
			"Grouping: expression expr",
			"Literal: value interface{}",
			"Logical: left expr, operator *token, right expr",
			"This: keyword *token",
			"Unary: operator *token, right expr",
			"Variable: name *token",
			"Function: params []*token, body []stmt",
		})
	}
	fmt.Println(out)
}

func generateAst(baseName string, types []string) string {
	out := "package internal\n\n"

	// Start base interface
	out += "type " + strings.ToLower(baseName) + " interface {\n"
	out += "\taccept(" + strings.ToLower(baseName) + "Visitor) R\n"
	out += "}\n\n"
	// End base interface

	// Start Visitor interface
	out += fmt.Sprintf("type %sVisitor interface {\n", strings.ToLower(baseName))
	for _, t := range types {
		typeDef := strings.Split(t, ":")
		name := strings.TrimSpace(typeDef[0])
		structType := strings.ToLower(string(name[0])) + name[1:] + baseName
		out += "\tvisit" + name + baseName + "(" + strings.ToLower(baseName) + " *" + structType + ") R\n"
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
	out := "type " + structName + " struct {\n"
	fieldArray := strings.Split(fields, ",")
	for _, field := range fieldArray {
		out += "\t" + strings.TrimSpace(field) + "\n"
	}
	out += "}\n\n"
	// End Structure Definition

	// Start Method Definition
	out += "func (s *" + structName + ") accept(visitor " + strings.ToLower(baseName) + "Visitor) R {\n"
	out += "\treturn visitor.visit" + name + baseName + "(s)\n"
	out += "}\n\n"
	// End Method Definition

	return out
}
