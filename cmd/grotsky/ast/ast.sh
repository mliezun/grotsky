#!/bin/sh

go run generate.go Stmt > ../../../internal/stmt.go
go run generate.go Expr > ../../../internal/expr.go
