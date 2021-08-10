# Grotsky
[![codecov](https://codecov.io/gh/mliezun/grotsky/branch/master/graph/badge.svg)](https://codecov.io/gh/mliezun/grotsky)
[![build](https://github.com/mliezun/grotsky/workflows/build/badge.svg)](https://github.com/mliezun/grotsky/actions?query=workflow%3Abuild)


Grotsky toy programming language. Made after reading the book Crafting Interpreters. You can also find clox and jlox implementations on my github projects.

Grotsky is inspired a little bit by go and python. Uses a C-like style with curly braces but no semicolons, includes some basic collections like lists and dicts. Also has the hability to listen to tcp ports, read environment variables and import modules.

## Overview

Get executable from [releases](https://github.com/mliezun/grotsky/releases/).

Run using `$ ./grotsky source.gr`

### Literals

- `strings`: "String A"
- `numbers`: 10, 10.04, 3.14, -1
- `booleans`: true, false
- `lists`: ["a", 1, 2]
- `dicts`: {"a": 1}

### Hello World

```js
io.println("Hello world!")
```

### Arithmetic Expressions

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

### Comparison & Logical Expressions

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

### Lists

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

### Dicts

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

### Conditionals

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

### Loops

#### While-Loop

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

### Functions and Closures

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
