package internal

import (
	"fmt"
	"testing"
)

type testPrinter struct {
	printed string
}

func (t *testPrinter) Println(a ...interface{}) (n int, err error) {
	for i, e := range a {
		if i != 0 {
			t.printed += " "
		}
		t.printed += fmt.Sprintf("%v", e)
	}
	t.printed += "\n"
	return 0, nil
}

func (t *testPrinter) Equals(p string) bool {
	if t.printed == p+"\n" {
		t.Reset()
		return true
	}
	return false
}

func (t *testPrinter) Reset() {
	t.printed = ""
}

func checkExpression(t *testing.T, exp string, result string) {
	source := "io.println(" + exp + ")"
	tp := &testPrinter{}
	RunSourceWithPrinter(source, tp)
	if !tp.Equals(result) {
		t.Errorf(`Should be equal to %s instead of %s`, result, tp.printed)
	}
}

func checkStatements(t *testing.T, code string, resultVar string, result string) {
	source := code + "\nio.println(" + resultVar + ")"
	tp := &testPrinter{}
	RunSourceWithPrinter(source, tp)
	if !tp.Equals(result) {
		t.Errorf(`Should be equal to %s instead of %s`, result, tp.printed)
	}
}

func TestExpressions(t *testing.T) {

	// Arithmethic
	{
		// Number
		checkExpression(t, "1", "1")

		// Negative
		checkExpression(t, "-1", "-1")

		// Add numbers
		checkExpression(t, "1 + 2 + 3", "6")

		// Subtract numbers
		checkExpression(t, "8 - 2", "6")

		// Multiply numbers
		checkExpression(t, "1 * 2 * 3", "6")

		// Divide numbers
		checkExpression(t, "12 / 2", "6")

		// Power numbers
		checkExpression(t, "2^2", "4")
	}

	// Logical
	{
		// 'true' literal
		checkExpression(t, "true", "true")

		// 'false' literal
		checkExpression(t, "false", "false")

		// not
		checkExpression(t, "not false", "true")
		checkExpression(t, "not true", "false")
		checkExpression(t, "not nil", "true")
		checkExpression(t, `not ""`, "true")
		checkExpression(t, `not 0`, "true")
		checkExpression(t, `not []`, "false")
		checkExpression(t, `not {}`, "false")

		// and
		checkExpression(t, "true and true", "true")
		checkExpression(t, "false and true", "false")
		checkExpression(t, "true and false", "false")
		checkExpression(t, "false and false", "false")

		// or
		checkExpression(t, "false or false", "false")
		checkExpression(t, "false or true", "true")
		checkExpression(t, "true or true", "true")
		checkExpression(t, "true or false", "true")
	}

	// Strings
	{
		// String literal
		checkExpression(t, `"test"`, "test")

		// String concat
		checkExpression(t, `"te" + "st"`, "test")
	}

	// Comparisons
	{
		// String Equality
		checkExpression(t, `"test" == "test"`, "true")
		checkExpression(t, `"test" != "test"`, "false")

		// Number Equality
		checkExpression(t, `2*2 == 2^3-4`, "true")
		checkExpression(t, `2*2 != 2^3-4`, "false")

		// Number gt
		checkExpression(t, `10 > 5`, "true")

		// Number lt
		checkExpression(t, `10 < 5`, "false")

		// Number gte
		checkExpression(t, `5 >= 5`, "true")
		checkExpression(t, `4 >= 5`, "false")

		// Number lte
		checkExpression(t, `5 <= 5`, "true")
		checkExpression(t, `10 <= 5`, "false")
	}

	// Lists
	{
		// List literalals
		checkExpression(t, "[]", "[]")
		checkExpression(t, "[1, 2, 3]", "[1 2 3]")
		checkExpression(t, `[["test", 2^4], not true, 1 < 2]`, "[[test 16] false true]")
		checkExpression(t, "[[1, 2], [3, 4]]", "[[1 2] [3 4]]")

		// List slicing
		checkExpression(t, "[1,2,3,4,5,6][1:][::2][0]", "2")
		checkExpression(t, "[1,2,3,4,5,6][:4][::3][1]", "4")
		checkExpression(t, "[1,2,3,4,5,6][1:5:2]", "[2 4]")
		checkExpression(t, "[1,2,3,4,5,6][1:5]", "[2 3 4 5]")
		checkExpression(t, "[1,2,3,4,5,6][1::2]", "[2 4 6]")
		checkExpression(t, "[1,2,3,4,5,6][:5:2]", "[1 3 5]")
	}

	// Dicts
	{
		// Dict literalals
		checkExpression(t, "{}", "map[]")
		checkExpression(t, `{1:2, "a":"b", 3: [1+2*3, "te" + "st"]}`, "map[1:2 3:[7 test] a:b]")

		// Dict Access
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[1]`, "map[a:3]")
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[1]["a"]`, "3")
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[3][0]`, "7")
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[3][1]`, "test")
	}
}

func TestStatements(t *testing.T) {
	// If-elif-else
	{
		checkStatements(t, `
		let i = 0
		if i == 100 begin
			i = 10
		elif i < 10
			i = 20
		else
			i = 100
		end
		`, "i", "20")

		checkStatements(t, `
		let i = 20
		if i == 100 begin
			i = 10
		elif i < 10
			i = 20
		else
			i = 100
		end`, "i", "100")

		checkStatements(t, `
		let i = 100
		if i == 100 begin
			i = 10
		elif i < 10
			i = 20
		else
			i = 100
		end`, "i", "10")
	}

	// While loop
	{
		checkStatements(t, `
		let i = 0
		while i*2 < 10 begin
			i = i + 1
		end
		`, "i", "5")
	}

	// For loop
	{
		checkStatements(t, `
		let x = 1
		for let i = 1; i <= 8; i = i+1 begin
			x = x * i
		end`, "x", "40320")

		checkStatements(t, `
		let x = 40320
		let u = 0
		for ; u < 10; u = u + 1 begin
			x = x - u
		end
		`, "x", "40275")

		// TODO: fix for init withouth LET token / problem caused by enhanced for
		/*
			checkStatements(t, `
			let x = 40320
			let u = 0
			for u=2*3; u < 10; u = u + 1 begin
				x = x - u
			end
			`, "x", "40275")
		*/

		checkStatements(t, `
		let x = 40275
		let arr = [1, 2, 3, 4]
		for el in arr begin
			x = x + el
		end`, "x", "40285")

		checkStatements(t, `
		let x = 40285
		let mat = [[1, 2], [3, 4]]
		for n, m in mat begin
			x = x + n + m
		end`, "x", "40295")

		checkStatements(t, `
		let x = 40295
		let dict = {1: 2, 3: 4}
		for key, val in dict begin
			x = x + key + val
		end`, "x", "40305")

		checkStatements(t, `
		let x = 40305
		let dict = {1: 2, 3: 4}
		for key in dict begin
			x = x + key
		end
		`, "x", "40309")
	}

	// Functions
	{
		checkStatements(t, `
		fn check() begin
			return 1
		end
		let i = check()
		`, "i", "1")

		checkStatements(t, `
		fn check(i) begin
			return i
		end
		let i = check(10)
		`, "i", "10")

		// TODO: fix returns / probably is because of return malfunction
		/*
			checkStatements(t, `
			fn fib(i) begin
				if i < 2 begin
					return i
				end
				return fib(i-1)+fib(i-2)
			end
			let f = fib(10)
			`, "i", "1")
		*/
	}
}
