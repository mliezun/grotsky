package internal

import (
	"fmt"
	"io"
	"strings"
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

func (t *testPrinter) Fprintf(w io.Writer, format string, a ...interface{}) (n int, err error) {
	return t.Println(fmt.Sprintf(format, a...))
}

func (t *testPrinter) Fprintln(w io.Writer, a ...interface{}) (n int, err error) {
	return t.Println(a...)
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

func checkExpression(t *testing.T, exp string, result ...string) {
	source := "io.println(" + exp + ")"
	tp := &testPrinter{}
	RunSourceWithPrinter("", source, tp)
	any := false
	for _, r := range result {
		if tp.Equals(r) {
			any = true
			break
		}
	}
	if !any {
		t.Errorf(
			"Error on: \n%s\n\tResult should be equal to %s instead of %s",
			exp,
			result,
			tp.printed,
		)
	}
}

func checkErrorMsg(t *testing.T, source string, errorMsg string, line int) {
	result := fmt.Sprintf("Runtime Error on line %d\n\t%s\n", line, errorMsg)

	tp := &testPrinter{}
	RunSourceWithPrinter("", source, tp)
	if !tp.Equals(result) {
		t.Errorf(
			"\nSource:\n----\n%s\n----\nExpected:\n----\n%s----\nFound:\n----\n%s----",
			source,
			result,
			tp.printed,
		)
	}
}

func checkStatements(t *testing.T, code string, resultVar string, result string) {
	source := code + "\nio.println(" + resultVar + ")"
	tp := &testPrinter{}
	RunSourceWithPrinter("", source, tp)
	if !tp.Equals(result) {
		t.Errorf(
			"Error on: \n%s\n\t%s should be equal to %s instead of %s",
			code,
			resultVar,
			result,
			tp.printed,
		)
	}
}

func checkLexer(t *testing.T, source string, line int, result ...string) {
	tp := &testPrinter{}
	RunSourceWithPrinter("", source, tp)
	compare := ""
	for i, r := range result {
		if i != 0 {
			compare += "\n"
		}
		compare += fmt.Sprintf("Error on line %d\n\t%s", line, r)
	}
	if !tp.Equals(compare) {
		t.Errorf(
			"\nExpected:\n%s\nEncountered:\n%s\n",
			compare,
			tp.printed,
		)
	}
}

func TestLexer(t *testing.T) {
	checkLexer(t, "1 ! 2", 1, errWrongBang.Error())

	checkLexer(t, "@", 1, errIllegalChar.Error())

	checkLexer(t, `"`, 1, errUnclosedString.Error())
}

func generateMaxParams() string {
	params := ""
	current := 'a'
	wraps := 1
	for i := 0; i <= maxFunctionParams+1; i++ {
		if i != 0 {
			params += ", "
		}
		params += strings.Repeat(string(current), wraps)
		current++
		if current == 'z'+1 {
			current = 'a'
			wraps++
		}
	}
	return params
}

func TestParser(t *testing.T) {
	// Function errors
	{
		params := generateMaxParams()
		checkLexer(t, "fn foo("+params+"){}", 1, errMaxParameters.Error())
		checkLexer(t, "foo("+params+")", 1, errMaxArguments.Error())
		checkLexer(t, "let foo = fn ("+params+")", 1, errMaxParameters.Error())
	}

	// For-loop error
	{
		checkLexer(t, `for 1; i < 2; i = i + 1 {}`, 1, errExpectedInit.Error(), errUndefinedExpr.Error())
	}

	// break,continue error
	{
		checkLexer(t, `break`, 1, errOnlyAllowedInsideLoop.Error())
		checkLexer(t, `continue`, 1, errOnlyAllowedInsideLoop.Error())
	}

	// Assignment error
	{
		checkLexer(t, `"asd" = 3`, 1, errUndefinedStmt.Error())
	}

	// Consume error
	{
		checkLexer(t, "class A{} let i = 2", 1, errExpectedNewline.Error())
	}

	// Synchronization
	{
		checkLexer(t, `
		"a" = 2
		class A {

		}
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		for let i = 0; i < 0; i = i + 1 {

		}
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		fn A () {

		}
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		if 2 < 1 {

		}
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		let a = 1
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		while 2 < 1 {

		}
		`, 2, errUndefinedStmt.Error())

		checkLexer(t, `
		"a" = 2
		return 2
		`, 2, errUndefinedStmt.Error())
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

		// Mod numbers
		checkExpression(t, "10 % 2", "0")

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
		checkExpression(t, `"test"`, `test`)
		checkExpression(t, `"
		Title
		body
		"`, `
		Title
		body
		`)
		checkExpression(t, "\r", ``)

		// String concat
		checkExpression(t, `"te" + "st"`, `test`)

		// String length
		checkExpression(t, `"test".length`, `4`)

		// String accessing
		checkExpression(t, `"test"[0]`, `t`)
		checkExpression(t, `"test"[0:2]`, `te`)
		checkExpression(t, `"longtest"[1:6:2]`, `oge`)
		checkExpression(t, `""[1:6:2]`, ``)
	}

	// Comparisons
	{
		// String Equality
		checkExpression(t, `"test" == "test"`, "true")
		checkExpression(t, `"test" != "test"`, "false")

		// String gt
		checkExpression(t, `"a" > "b"`, "false")

		// String lt
		checkExpression(t, `"a" < "b"`, "true")

		// String gte
		checkExpression(t, `"a" >= "a"`, "true")
		checkExpression(t, `"a" >= "b"`, "false")

		// String lte
		checkExpression(t, `"a" <= "a"`, "true")
		checkExpression(t, `"b" <= "a"`, "false")

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

		// Grouping
		checkExpression(t, `(5 <= 5) and (not true or ((1*(1+4)) == 5))`, "true")
	}

	// Lists
	{
		// List literalals
		checkExpression(t, "[]", "[]")
		checkExpression(t, "[].length", "0")
		checkExpression(t, "[1.0, 2.0, 3.0]", "[1, 2, 3]")
		checkExpression(t, "[1.0, 2.0, 3.0].length", "3")
		checkExpression(t, `[["test", 2^4], not true, 1 < 2]`, `[["test", 16], false, true]`)
		checkExpression(t, "[[1, 2], [3, 4]]", "[[1, 2], [3, 4]]")

		// List slicing
		checkExpression(t, "[][1:][::2][0]", "[]")
		checkExpression(t, "[1,2,3,4,5,6][1:][::2][0]", "2")
		checkExpression(t, "[1,2,3,4,5,6][:4][::3][1]", "4")
		checkExpression(t, "[1,2,3,4,5,6][1:5:2]", "[2, 4]")
		checkExpression(t, "[1,2,3,4,5,6][1:5]", "[2, 3, 4, 5]")
		checkExpression(t, "[1,2,3,4,5,6][1::2]", "[2, 4, 6]")
		checkExpression(t, "[1,2,3,4,5,6][:5:2]", "[1, 3, 5]")

		// List operations
		checkExpression(t, "[1,2,3] + [4,5,6]", "[1, 2, 3, 4, 5, 6]")
		checkExpression(t, "-[1,1,1,1,1,1]", "[1]")
		checkExpression(t, "[1] == [1]", "false")
		checkExpression(t, "[1] != [1]", "true")
		checkExpression(t, "[1,2,3] - [2,3]", "[1]")

		// List access
		checkExpression(t, `[1,2,3,4,5,6,7,8,9,10,11,12,13,14][:20:0][0:10:100]`, "[]")
		checkExpression(t, `[1,2][1:20]`, "[2]")
	}

	// Dicts
	{
		// Dict literalals
		checkExpression(t, "{}", "{}")
		checkExpression(t, "{}.length", "0")
		checkExpression(t, `{0: 0, 1: 1}`, `{0: 0, 1: 1}`, `{1: 1, 0: 0}`)
		checkExpression(t, `{0: 0, 1: 1}.length`, `2`)

		// Dict Access
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[1]`, `{"a": 3}`)
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[1]["a"]`, "3")
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[3][0]`, "7")
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[3][1]`, `test`)
		checkExpression(t, "{}[0]", "{}")

		// Dict operations
		checkExpression(t, `{1: 2} + {1: 4}`, "{1: 4}")
		checkExpression(t, `{1: 2} == {1: 2}`, "false")
		checkExpression(t, `{1: 2} != {1: 2}`, "true")
	}

	// Function expressions
	{
		// Func literalals
		checkExpression(t, "fn () nil", "<fn anonymous>")
		checkExpression(t, "(fn () nil)()", "<nil>")
	}

	// Nil handling
	{
		// Eq-Neq operations with nil
		checkExpression(t, "nil != 0", "true")
		checkExpression(t, "nil != nil", "false")
		checkExpression(t, "nil == nil", "true")
		checkExpression(t, "nil == [nil]", "false")
		checkExpression(t, "nil == [nil][0]", "true")
		checkExpression(t, `"asd" != nil`, "true")
		checkExpression(t, `"" == nil`, "false")
		checkExpression(t, `{"asd": 1} != nil`, "true")
		checkExpression(t, `{"asd": nil}["asd"] == nil`, "true")
	}
}

func TestRuntimeErrors(t *testing.T) {
	// Expression errors
	{
		// Subtract number with non-number
		checkErrorMsg(t, `1 - "B"`, fmt.Sprintf("%s: -", errExpectedNumber.Error()), 1)

		// Binary expression undefined
		checkErrorMsg(t, `"A" - "B"`, fmt.Sprintf("%s: -", errUndefinedOp.Error()), 1)

		// Unary expression undefined
		checkErrorMsg(t, `-"B"`, fmt.Sprintf("%s: -", errUndefinedOp.Error()), 1)

		// Call not callable
		checkErrorMsg(t, `"B"()`, fmt.Sprintf("%s: )", errOnlyFunction.Error()), 1)

		// Access on string
		checkErrorMsg(t, `"B".prop`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 1)

		// Set on string
		checkErrorMsg(t, `"B".prop = 1`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 1)

		// Operate on string + non-string
		checkErrorMsg(t, `"B" + 1`, fmt.Sprintf("%s: +", errExpectedString.Error()), 1)

		// Wrong number of arguments
		checkErrorMsg(t, `(fn (a, b) a+b)()`, fmt.Sprintf("%s: )", errInvalidNumberArguments.Error()), 1)

		// Get expr on non-object
		checkErrorMsg(t, `(fn (a, b) a+b).length`, fmt.Sprintf("%s: length", errExpectedObject.Error()), 1)

		// Set expr on non-object
		checkErrorMsg(t, `(fn (a, b) a+b).length = 1`, fmt.Sprintf("%s: length", errExpectedObject.Error()), 1)

		// Wrong slicing
		checkErrorMsg(t, `[1,2,3,4,5,6][0:3:]`, fmt.Sprintf("%s: :", errExpectedStep.Error()), 1)

		// Wrong slicing type
		checkErrorMsg(t, `[1,2,3,4,5,6]["0":]`, fmt.Sprintf("%s: [", errOnlyNumbers.Error()), 1)

		// Cannot slice a number
		checkErrorMsg(t, `2[0]`, fmt.Sprintf("%s: [", errInvalidAccess.Error()), 1)

		// Get prop from list
		checkErrorMsg(t, `[].prop`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 1)

		// Set prop list
		checkErrorMsg(t, `[].prop = 1`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 1)

		// Operate on list + non-list
		checkErrorMsg(t, `[] + ""`, fmt.Sprintf("%s: +", errExpectedList.Error()), 1)

		// Undefined op list
		checkErrorMsg(t, `[] * []`, fmt.Sprintf("%s: *", errUndefinedOp.Error()), 1)

		// Get prop from dict
		checkErrorMsg(t, `let a = {}.prop`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 1)

		// Set prop dict
		checkErrorMsg(t, `let a = {}
		a.prop = 1`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 2)

		// Operate on dict + non-dict
		checkErrorMsg(t, `let a = {} + ""`, fmt.Sprintf("%s: +", errExpectedDict.Error()), 1)

		// Undefined dict operation
		checkErrorMsg(t, `let a = {} * {}`, fmt.Sprintf("%s: *", errUndefinedOp.Error()), 1)

		// Undefined operation on nil
		checkErrorMsg(t, `nil <= nil`, fmt.Sprintf("%s: <=", errUndefinedOp.Error()), 1)
		checkErrorMsg(t, `nil + ""`, fmt.Sprintf("%s: +", errUndefinedOp.Error()), 1)
		checkErrorMsg(t, `[] + nil`, fmt.Sprintf("%s: +", errUndefinedOp.Error()), 1)
	}

	// Statement errors
	{
		// Undefined property on number
		checkErrorMsg(t, `
		let a = 1
		a.isnumber
		`, fmt.Sprintf("%s: isnumber", errUndefinedProp.Error()), 3)

		// Number object is read only
		checkErrorMsg(t, `
		let a = 1
		a.isnumber = false
		`, fmt.Sprintf("%s: isnumber", errReadOnly.Error()), 3)

		// Undefined variable
		checkErrorMsg(t, `let a = b`, fmt.Sprintf("%s: b", errUndefinedVar.Error()), 1)

		// Undefined variable assignment
		checkErrorMsg(t, `a = 1`, fmt.Sprintf("%s: a", errUndefinedVar.Error()), 1)

		// Access on booleans
		checkErrorMsg(t, `
		let a = true
		a.isbool
		`, fmt.Sprintf("%s: isbool", errUndefinedProp.Error()), 3)

		// Set on booleans
		checkErrorMsg(t, `
		let a = true
		a.isbool = false
		`, fmt.Sprintf("%s: isbool", errReadOnly.Error()), 3)

		// Operate on booleans
		checkErrorMsg(t, `
		let a = true
		let b = false
		a + b
		`, fmt.Sprintf("%s: +", errUndefinedOp.Error()), 4)

		// Wrong array destructuring
		checkErrorMsg(t, `
			for a, b, c in [[1,2]] {
				io.println(a+b+c)
			}
			`, fmt.Sprintf("%s: for", errWrongNumberOfValues.Error()), 2)

		// Cannot unpack
		checkErrorMsg(t, `
			for a, b, c in ["abc"] {
				io.println(a+b+c)
			}
			`, fmt.Sprintf("%s: for", errCannotUnpack.Error()), 2)

		// Cannot unpack dict with more than 2 identifiers
		checkErrorMsg(t, `
			for a, b, c in {"a": ["abc"]} {
				io.println(a+b+c)
			}
			`, fmt.Sprintf("%s: for", errExpectedIdentifiersDict.Error()), 2)

		// Only collections are iterable
		checkErrorMsg(t, `
			for a, b, c in "abc" {
				io.println(a+b+c)
			}
			`, fmt.Sprintf("%s: for", errExpectedCollection.Error()), 2)

		// Error on dict access
		checkErrorMsg(t, `
			let abc = {"a": 1, "b": 2, "c": 3}
			abc[1:2:3]
			`, fmt.Sprintf("%s: [", errExpectedKey.Error()), 3)
		checkErrorMsg(t, `
			let abc = {"a": 1, "b": 2, "c": 3}
			abc[:2]
			`, fmt.Sprintf("%s: [", errExpectedKey.Error()), 3)
		checkErrorMsg(t, `
			let abc = {"a": 1, "b": 2, "c": 3}
			abc[::2]
			`, fmt.Sprintf("%s: [", errExpectedKey.Error()), 3)

		// Inheritance from non-class
		checkErrorMsg(t, `
		let C = "C"
		class A < C {
		}
		`, fmt.Sprintf("%s: A", errExpectedClass.Error()), 3)

		// Inheritance from non-class
		checkErrorMsg(t, `
		class C {
		}
		class A < C {
			get(a) {
				return super.get(a)
			}
		}
		A().get(1)
		`, fmt.Sprintf("%s: get", errMethodNotFound.Error()), 6)

		// Get prop from class
		checkErrorMsg(t, `
		class A {
		}
		A.prop
		`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 4)

		// Set prop from class
		checkErrorMsg(t, `
		class A {
		}
		A.prop = 1
		`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 4)

		// Operate on class
		checkErrorMsg(t, `
		class A {
		}
		A + A
		`, fmt.Sprintf("%s: +", errUndefinedOp.Error()), 4)

		// Error on constructor
		checkErrorMsg(t, `
		class A {
			init() {
			}
		}
		A(1)
		`, fmt.Sprintf("%s: )", errInvalidNumberArguments.Error()), 6)

		// Undefined object property
		checkErrorMsg(t, `
		class A {
		}
		A().get
		`, fmt.Sprintf("%s: get", errUndefinedProp.Error()), 4)

		// Undefined object operator
		checkErrorMsg(t, `
		class A {
		}
		A() + A()
		`, fmt.Sprintf("%s: +", errUndefinedOperator.Error()), 4)
	}
}

func TestGlobals(t *testing.T) {
	// Print globals
	{
		checkExpression(t, `io`, "<instance native>")
		checkExpression(t, `io.println`, "<fn native>")
	}

	// Get-Set globals
	{
		checkErrorMsg(t, `io.miguel = 2`, errReadOnly.Error()+": miguel", 1)
	}
}

func TestStatements(t *testing.T) {
	// Comment
	{
		checkStatements(t, `
		# This is a "comment"
		let i = 0
		`, "i", "0")
	}

	// If-elif-else
	{
		checkStatements(t, `
		let i = 0
		if i == 100 {
			i = 10
		} elif i < 10 {
			i = 20
		} else {
			i = 100
		}
		`, "i", "20")

		checkStatements(t, `
		let i = 20
		if i == 100 {
			i = 10
		} elif i < 10 {
			i = 20
		} else {
			i = 100
		}`, "i", "100")

		checkStatements(t, `
		let i = 100
		if i == 100 {
			i = 10
		} elif i < 10 {
			i = 20
		} else {
			i = 100
		}`, "i", "10")
	}

	// While loop
	{
		checkStatements(t, `
		let i = 0
		while i*2 < 10 {
			i = i + 1
		}
		`, "i", "5")
	}

	// For loop
	{
		checkStatements(t, `
		let x = 1
		for let i = 1; i <= 8; i = i+1 {
			x = x * i
		}`, "x", "40320")

		checkStatements(t, `
		let x = 40320
		let u = 0
		for ; u < 10; u = u + 1 {
			x = x - u
		}
		`, "x", "40275")

		checkStatements(t, `
		let x = 40275
		let arr = [1, 2, 3, 4]
		for el in arr {
			x = x + el
		}`, "x", "40285")

		checkStatements(t, `
		let x = 40285
		let mat = [[1, 2], [3, 4]]
		for n, m in mat {
			x = x + n + m
		}`, "x", "40295")

		checkStatements(t, `
		let x = 40295
		let dict = {1: 2, 3: 4}
		for key, val in dict {
			x = x + key + val
		}`, "x", "40305")

		checkStatements(t, `
		let x = 40305
		let dict = {1: 2, 3: 4}
		for key in dict {
			x = x + key
		}
		`, "x", "40309")
	}

	// Functions
	{
		checkStatements(t, `
		fn nilCheck() {
			return
		}
		let i = nilCheck()
		`, "i", "<nil>")

		// simplified function
		checkStatements(t, `
		fn one() 1
		let i = one()
		`, "i", "1")

		// assign lambda function
		checkStatements(t, `
		let one = fn () {
			return 1
		}
		let i = one()
		`, "i", "1")

		checkStatements(t, `
		fn check() {
			return 1
		}
		let i = check()
		`, "i", "1")

		checkStatements(t, `
		fn check(i) {
			return i
		}
		let i = check(10)
		`, "i", "10")

		checkStatements(t, `
		fn fib(i) {
			if i == 0 {
				return 0
			} elif i == 1 {
				return 1
			} else {
				return fib(i-1)+fib(i-2)
			}
		}
		let f = fib(10)
		`, "f", "55")

		checkStatements(t, `
		fn count(i) {
			while true {
				i = i - 1
				if i < 0 {
					return i
				}
			}
			return i
		}
		let f = count(10)
		`, "f", "-1")

		checkStatements(t, `
		fn count(i) {
			while true {
				return i
			}
			return i
		}
		let f = count(10)
		`, "f", "10")

		checkStatements(t, `
		fn count(i) {
			for let n = 0; n < 1; n = n + 1 {
				return n
			}
			return i
		}
		let f = count(10)
		`, "f", "0")

		checkStatements(t, `
		fn testBreakContinue() {
			let i = 0
			while true {
				i = i + 1
				if i % 2 == 0 {
					continue
				}
				if i >= 10 {
					break
				}
			}
			return i
		}
		let f = testBreakContinue()
		`, "f", "11")

		checkStatements(t, `
		fn firstEl(arr) {
			for e in arr {
				return e
			}
		}
		let f = firstEl([3,4,5])
		`, "f", "3")

		checkStatements(t, `
		fn firstKey(dict) {
			for key in dict {
				return key
			}
		}
		let f = firstKey({1:2})
		`, "f", "1")

		// Print function
		checkStatements(t, `
		fn ff() {
		}
		`, "ff", "<fn ff>")
	}

	// Classes
	{
		// Check simple object
		checkStatements(t, `
		class Pan {
			init () {
				this.pan = 1
			}
		}`, "Pan().pan", "1")

		// Check parent constructor
		checkStatements(t, `
		class Food {
			init () {
				this.msg = "good"
			}
		}
		class Pan < Food {
			init () {
				super.init()
			}
		}`, "Pan().msg", `good`)

		// Check method inheritance
		checkStatements(t, `
		class Food {
			eat () {
				this.msg = "eating"
			}
		}
		class Pan < Food {
		}
		let bread = Pan()
		bread.eat()
		`, "bread.msg", `eating`)

		// Class methods
		checkStatements(t, `
		class Container {
			class get(a) {
				return a
			}
		}
		`, "Container.get(1)", "1")

		// Operator overload
		checkStatements(t, `
		class Operate {
			init (val) {
				this.val = val
			}

			add (o) {
				return Operate(o.val + this.val)
			}
		}
		let a = Operate(1)
		let b = Operate(2)
		let c = a + b
		`, "c.val", "3")

		// Print object
		checkStatements(t, `
		class Operate {
			init (val) {
				this.val = val
			}

			add (o) {
				return Operate(o.val + this.val)
			}
		}
		let a = Operate(1)
		`, "a", "<instance <class Operate>>")

		// Print class
		checkStatements(t, `
		class B {
		}
		class A < B {
		}
		`, "A", "<class A extends B>")

		// check super constructor
		checkStatements(t, `
		class B {
			init() {
				this.msg = "ok"
			}
		}
		class A < B {
			init() {
				super()
			}
		}
		let a = A()
		`, "a.msg", "ok")
	}

	// Assignments to colections
	{
		// Assign to list
		checkStatements(t, `
		let list = [1, 2, 3]
		list[0] = 10
		`, "list[0]", "10")

		// Assign to dict
		checkStatements(t, `
		let dict = {
			"a": 1,
			"b": 2
		}
		dict["c"] = 3`, `dict["c"]`, "3")

		// Assign to list inside list
		checkStatements(t, `
		let list = [[1], 2, 3]
		list[0][0] = 10
		`, "list[0][0]", "10")

		// Assign to list inside dict
		checkStatements(t, `
		let dictlist = {"a": [1, 2, 3]}
		dictlist["a"][0] = 10
		`, `dictlist["a"][0]`, "10")

		// Assign to dict inside list
		checkStatements(t, `
		let listdict = [1, 2, 3, {"a": 1}]
		listdict[3]["b"] = 2
		`, `listdict[3]["b"]`, "2")

		// Assign to dict inside dict
		checkStatements(t, `
		let dictdict = {"a": {"A": 1}}
		dictdict["a"]["B"] = 2
		`, `dictdict["a"]["B"]`, "2")

		// Assign to dict inside class
		checkStatements(t, `
		class A {
			init() {
				this.dict = {}
			}

			do() {
				this.dict["a"] = 1
			}
		}
		let a = A()
		a.do()`, `a.dict["a"]`, "1")
	}

	// Types
	{
		checkStatements(t, `
		io.println(type(""))
		io.println(type(1))
		io.println(type([]))
		io.println(type({}))
		class A {
			get() {}
		}
		io.println(type(A))
		io.println(type(A()))
		io.println(type(A().get))
		io.println(type(nil))
		`, "type(true)", "string\nnumber\nlist\ndict\nclass\nobject\nfunction\nnil\nbool")
	}
}
