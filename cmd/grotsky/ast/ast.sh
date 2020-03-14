#!/bin/sh

go run ast.go Stmt > ../../../internal/stmt.go
go run ast.go Expr > ../../../internal/expr.go

echo "AST Generated"
