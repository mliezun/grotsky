# Grotsky
[![codecov](https://codecov.io/gh/mliezun/grotsky/branch/master/graph/badge.svg)](https://codecov.io/gh/mliezun/grotsky)
[![build](https://github.com/mliezun/grotsky/workflows/build/badge.svg)](https://github.com/mliezun/grotsky/actions?query=workflow%3Abuild)


Grotsky is a toy programming language. Implemented in Rust as a Bytecode register-based vm interpreter. Made after reading the book Crafting Interpreters. You can also find clox and jlox implementations under my github projects.

Grotsky is inspired a little bit by go, python and javascript. Uses a C-like style syntax with curly braces but no semicolons, includes some basic collections like lists and dicts. Also has the hability to listen to tcp ports, read environment variables and import modules.

It has the ability to compile to bytecode and also to embed scripts into a distributable binary.

# Table of Contents
1. [Usage](#usage)
    - [Run Scripts](#run-scripts)
    - [Compile Scripts](#compile-scripts)
    - [Embed Scripts](#embed-scripts)
2. [Literals](#literals)
3. [Print: Hello World](#print-hello-world)
4. [Comments](#comments)
5. [Arithmetic Expressions](#arithmetic-expressions)
6. [Comparison and Logical Expressions](#comparison-and-logical-expressions)
7. [Lists](#lists)
8. [Dicts](#dicts)
9. [Conditionals](#conditionals)
10. [Loops](#loops)
    - [While Loop](#while-loop)
    - [Classic For Loop](#classic-for-loop)
    - [Enhanced For Loop](#enhanced-for-loop)
        - [Iterate List](#iterate-list)
        - [Iterate Dict](#iterate-dict)
        - [Unpacked List of Lists](#unpacked-list-of-lists)
11. [Functions and Closures](#functions-and-closures)
12. [Classes](#classes)
    - [Simple Class](#simple-class)
    - [Superclasses](#superclasses)
    - [Magic Methods](#magic-methods)
13. [Modules](#modules)
14. [Net Utils](#net-utils)
    - [TCP Socket](#tcp-socket)
15. [ENV Variables](#env-variables)
16. [Try-Catch](#try-catch)

## Usage

Get executable from [releases](https://github.com/mliezun/grotsky/releases/).

Run from command line:

```
$ ./grotsky
Usage:
    grotsky [script.gr | bytecode.grc]
    grotsky compile script.gr
    grotsky embed bytecode.grc
```

### Run Scripts

Create a Grotsky script, which consists of a text file with the `.gr` extension.

Then it can be executed by running the following command:

```
$ ./grotsky script.gr
```

### Compile Scripts

The Grotsky interpreter provides the ability to compile scripts to bytecode.

```
$ ./grotsky compile script.gr
```

Generates a new file called `script.grc` located alongside the input file.

The compiled file can then be executed by running:

```
$ ./grotsky script.grc
```

### Embed Scripts

First, the input script needs to be [compiled](#compile-scripts).

Then it can be embbeded into an executable binary that only runs that script.

```
$ ./grotsky embed script.grc
```

The resulting binary with the embedded code will be located alongside the input file and named with the `.exe` extension, for the previous example the file would be `script.exe`.

> NOTE: Currently Grotsky only supports embedding a single file, which means importing modules might not work as expected.

## Literals

- `strings`: "String A"
- `numbers`: 10, 10.04, 3.14, -1
- `booleans`: true, false
- `lists`: ["a", 1, 2]
- `dicts`: {"a": 1}

## Print: Hello World

```js
io.println("Hello world!")
```

## Comments

```js
# Comments start with '#'
```

## Arithmetic Expressions

```js
io.println(
    1+(2+5*10)/2,
    2^4
)
```

Outputs
```
27 16
```

## Comparison and Logical Expressions

```js
io.println(
    true and (false or true),
    1 > 2 and 5 <= 10
)
```

Outputs:
```
true false
```

## Lists

```js
let list = [
    "a",
    "b",
    "c",
    "d",
    "e"
]

io.println(list)
```

Outputs:
```
["a", "b", "c", "d", "e"]
```

## Dicts

```js
let dict = {
    "a": 1,
    "b": 2,
    "c": 3
}

io.println(dict)
```

Outputs:
```
{"c": 3, "a": 1, "b": 2}
```

## Conditionals

```js
let a = 10
if a > 10 {
    io.println("a is bigger than ten")
} elif a > 5 {
    io.println("a is less than or equal to ten and bigger than five")
} else {
    io.println("a is less than or equal to five")
}
```

Outputs:
```
a is less than or equal to ten and bigger than five
```

## Loops

### While Loop

```
let str = ""
while True {
    if str.length == 10 {
        break
    }
    str = str + "a"
}
io.println(str)
```

Outputs:
```
aaaaaaaaaa
```

### Classic For Loop

```js
let list = []
for let i = 0; i < 10; i = i + 1 {
    list = list + [i]
}
io.println(list)
```

Outputs:
```
[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
```

### Enhanced For-Loop

#### Iterate List
```js
let list = [0, 1, 2, 3, 4]
for i in list {
    io.println(i)
}
```

Outputs:
```
0
1
2
3
4
```

#### Iterate Dict
```js
let dict = {
    "a": 1,
    "b": 2,
    "c": 3
}
for k, v in dict {
    io.println(k, v)
}
```

Outputs:
```
b 2
c 3
a 1
```

#### Unpacked List of Lists

```js
let listOfLists = [
    ["a", 1, 3],
    ["b", 2, 4],
    ["c", 3, 5]
]
for x, y, z in listOfLists {
    io.println(x, y, z)
}
```

Outputs:
```
a 1 3
b 2 4
c 3 5
```

## Functions and Closures

```js
fn makeCounter() {
    let n = -1
    fn wrap() {
        n = n + 1
        return n
    }
    return wrap
}

let counter = makeCounter()

io.println(counter())
io.println(counter())
io.println(counter())
```

Outputs:
```
1
2
3
```

## Classes

### Simple Class

```js
class A {
    init(n) {
        this.n = n
    }

    print() {
        io.println("N:", this.n)
    }
}

let a = A(10)
a.print()
```

Outputs:
```
N: 10
```

### Superclasses

```js
class A {
    print() {
        io.println("Printing:", this.text)
    }

    appendToText(text) {
        this.text = this.text + text
    }
}

class B < A { # Inherits from A
    init() {
        this.text = ""
    }

    appendToText(text) {
        if this.text.length + text.length < 5 {
            super.appendToText(text) # Call to method on superclass
        }
    }
}

let b = B()
b.appendToText("Hello!") # str.length > 5
b.print()
b.appendToText("Hell")
b.print()
```

Outputs:
```
Printing: 
Printing: Hell
```

### Magic Methods

Available magic methods: add, sub, div, mod, mul, pow, neg, eq, neq, lt, lte, gt, gte.

```js
class Magic {
    init(value) {
        this.value = value
    }

    add(other) {
        # Addition as in result = this + other
        this.value = this.value + other.value
    }

    print() {
        io.println("Magic is:", this.value)
    }
}

let a = Magic(1)
a + Magic(2)
a.print()
```

Outputs:
```
Magic is: 3
```

## Modules

File `utils.gr`:
```js
fn map(list, mapFn) {
    let out = []
    for e in list {
        out = out + [mapFn(e)]
    }
    return out
}
```

Usage:
```js
let utils = import("utils.gr")
let list = [0, 1, 2, 3, 4]
io.println(utils.map(list, fn (e) 2 ^ e))
```

Outputs:
```js
[1, 2, 4, 8, 16]
```

## Net utils

### TCP Socket

```js
let socket = net.listenTcp("127.0.0.1", "6500")
while true {
    let conn = socket.accept()
    io.println("Received connection from:", conn.address())
    conn.write("ok\n")
    conn.close()
}
```

Try from console:
```
$ telnet localhost 6500
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
ok
Connection closed by foreign host.
```

Outputs:
```
Received connection from: 127.0.0.1:52238
```

## Env Variables

```js
let lang = env.Get("LANG")
io.println(lang)
```

Outputs:
```
en_US.UTF-8
```

## Try-Catch

```js
try {
    let utils = import("utils.gr")
} catch err {
    io.println(err)
}
```

Outputs:
```
open utils.gr: no such file or directory
```
