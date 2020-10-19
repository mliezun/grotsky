package internal

import (
	"fmt"
	"io"
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
	RunSourceWithPrinter(source, tp)
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
	RunSourceWithPrinter(source, tp)
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
	RunSourceWithPrinter(source, tp)
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
		checkExpression(t, `"test"`, `"test"`)

		// String concat
		checkExpression(t, `"te" + "st"`, `"test"`)
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
		checkExpression(t, `{1: {"a": 3}, 3: [1+2*3, "te" + "st"]}[3][1]`, `"test"`)

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
}

func TestRuntimeErrors(t *testing.T) {
	// Expression errors
	{
		// Binary expression undefined
		checkErrorMsg(t, `"A" - "B"`, fmt.Sprintf("%s: -", errUndefinedOp.Error()), 1)

		// Unary expression undefined
		checkErrorMsg(t, `-"B"`, fmt.Sprintf("%s: -", errUndefinedOp.Error()), 1)

		// Call not callable
		checkErrorMsg(t, `"B"()`, fmt.Sprintf("%s: )", errOnlyFunction.Error()), 1)

		// Wrong number of arguments
		checkErrorMsg(t, `(fn (a, b) a+b)()`, fmt.Sprintf("%s: )", errInvalidNumberArguments.Error()), 1)

		// Get expr on non-object
		checkErrorMsg(t, `(fn (a, b) a+b).length`, fmt.Sprintf("%s: length", errExpectedObject.Error()), 1)

		// Set expr on non-object
		checkErrorMsg(t, `(fn (a, b) a+b).length = 1`, fmt.Sprintf("%s: length", errExpectedObject.Error()), 1)

		// Can only access collections
		checkErrorMsg(t, `"abc"[0]`, fmt.Sprintf("%s: [", errInvalidAccess.Error()), 1)

		// Wrong slicing
		checkErrorMsg(t, `[1,2,3,4,5,6][0:3:]`, fmt.Sprintf("%s: :", errExpectedStep.Error()), 1)

		// Wrong slicing type
		checkErrorMsg(t, `[1,2,3,4,5,6]["0":]`, fmt.Sprintf("%s: [", errOnlyNumbers.Error()), 1)

		// Get prop from list
		checkErrorMsg(t, `[].prop`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 1)

		// Set prop list
		checkErrorMsg(t, `[].prop = 1`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 1)

		// Operate on list + non-list
		checkErrorMsg(t, `[] + ""`, fmt.Sprintf("%s: +", errExpectedList.Error()), 1)

		// Undefined op list
		checkErrorMsg(t, `[] * []`, fmt.Sprintf("%s: *", errUndefinedOp.Error()), 1)

		// Get prop from dict
		checkErrorMsg(t, `{}.prop`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 1)

		// Set prop dict
		checkErrorMsg(t, `{}.prop = 1`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 1)

		// Operate on dict + non-dict
		checkErrorMsg(t, `{} + ""`, fmt.Sprintf("%s: +", errExpectedDict.Error()), 1)

		// Undefined dict operation
		checkErrorMsg(t, `{} * {}`, fmt.Sprintf("%s: *", errUndefinedOp.Error()), 1)
	}

	// Statement errors
	{
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
			for a, b, c in [[1,2]] begin
				io.println(a+b+c)
			end
			`, fmt.Sprintf("%s: for", errWrongNumberOfValues.Error()), 2)

		// Cannot unpack
		checkErrorMsg(t, `
			for a, b, c in ["abc"] begin
				io.println(a+b+c)
			end
			`, fmt.Sprintf("%s: for", errCannotUnpack.Error()), 2)

		// Cannot unpack dict with more than 2 identifiers
		checkErrorMsg(t, `
			for a, b, c in {"a": ["abc"]} begin
				io.println(a+b+c)
			end
			`, fmt.Sprintf("%s: for", errExpectedIdentifiersDict.Error()), 2)

		// Only collections are iterable
		checkErrorMsg(t, `
			for a, b, c in "abc" begin
				io.println(a+b+c)
			end
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
		class A < C begin
		end
		`, fmt.Sprintf("%s: A", errExpectedClass.Error()), 3)

		// Inheritance from non-class
		checkErrorMsg(t, `
		class C begin
		end
		class A < C begin
			get(a) begin
				return super.get(a)
			end
		end
		A().get(1)
		`, fmt.Sprintf("%s: get", errMethodNotFound.Error()), 6)

		// Get prop from class
		checkErrorMsg(t, `
		class A begin
		end
		A.prop
		`, fmt.Sprintf("%s: prop", errUndefinedProp.Error()), 4)

		// Set prop from class
		checkErrorMsg(t, `
		class A begin
		end
		A.prop = 1
		`, fmt.Sprintf("%s: prop", errReadOnly.Error()), 4)

		// Operate on class
		checkErrorMsg(t, `
		class A begin
		end
		A + A
		`, fmt.Sprintf("%s: +", errUndefinedOp.Error()), 4)

		// Error on constructor
		checkErrorMsg(t, `
		class A begin
			init() begin
			end
		end
		A(1)
		`, fmt.Sprintf("%s: )", errInvalidNumberArguments.Error()), 6)
	}
}

func TestGlobals(t *testing.T) {
	// Print globals
	{
		checkExpression(t, `io`, "<instance native>")
		checkExpression(t, `io.println`, "<fn native>")

		checkExpression(t, `http`, "<instance native>")
		checkExpression(t, `http.handler`, "<fn native>")
		checkExpression(t, `http.listen`, "<fn native>")
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
		fn nilCheck() begin
			return
		end
		let i = nilCheck()
		`, "i", "<nil>")

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

		checkStatements(t, `
		fn fib(i) begin
			if i == 0 begin
				return 0
			elif i == 1
				return 1
			else
				return fib(i-1)+fib(i-2)
			end
		end
		let f = fib(10)
		`, "f", "55")

		checkStatements(t, `
		fn count(i) begin
			while true begin
				i = i - 1
				if i < 0 begin
					return i
				end
			end
			return i
		end
		let f = count(10)
		`, "f", "-1")

		checkStatements(t, `
		fn count(i) begin
			while true begin
				return i
			end
			return i
		end
		let f = count(10)
		`, "f", "10")

		checkStatements(t, `
		fn count(i) begin
			for let n = 0; n < 1; n = n + 1 begin
				return n
			end
			return i
		end
		let f = count(10)
		`, "f", "0")

		checkStatements(t, `
		fn firstEl(arr) begin
			for e in arr begin
				return e
			end
		end
		let f = firstEl([3,4,5])
		`, "f", "3")

		checkStatements(t, `
		fn firstKey(dict) begin
			for key in dict begin
				return key
			end
		end
		let f = firstKey({1:2})
		`, "f", "1")

		// Print function
		checkStatements(t, `
		fn ff() begin
		end
		`, "ff", "<fn ff>")
	}

	// Classes
	{
		// Check simple object
		checkStatements(t, `
		class Pan begin
			init () begin
				this.pan = 1
			end
		end`, "Pan().pan", "1")

		// Check parent constructor
		checkStatements(t, `
		class Food begin
			init () begin
				this.msg = "good"
			end
		end
		class Pan < Food begin
			init () begin
				super.init()
			end
		end`, "Pan().msg", `"good"`)

		// Check method inheritance
		checkStatements(t, `
		class Food begin
			eat () begin
				this.msg = "eating"
			end
		end
		class Pan < Food begin
		end
		let bread = Pan()
		bread.eat()
		`, "bread.msg", `"eating"`)

		// Class methods
		checkStatements(t, `
		class Container begin
			class get(a) begin
				return a
			end
		end
		`, "Container.get(1)", "1")

		// Operator overload
		checkStatements(t, `
		class Operate begin
			init (val) begin
				this.val = val
			end

			add (o) begin
				return Operate(o.val + this.val)
			end
		end
		let a = Operate(1)
		let b = Operate(2)
		let c = a + b
		`, "c.val", "3")

		// Print object
		checkStatements(t, `
		class Operate begin
			init (val) begin
				this.val = val
			end

			add (o) begin
				return Operate(o.val + this.val)
			end
		end
		let a = Operate(1)
		`, "a", "<instance <class Operate>>")

		// Print class
		checkStatements(t, `
		class B begin
		end
		class A < B begin
		end
		`, "A", "<class A extends B>")
	}
}
