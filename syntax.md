# Grotsky Syntax

## Expressions

#### Literals

Booleans
```
true
false
```

Numbers
```
1
1.3
5.66928
```

Strings
```
"I am a string"
```

Lists
```
[1, 2, 3, 1.2, "strings", ["lists"]]
```

Dictionaries
```
{
    "a": "b",
    "c": 1,
    "d": 1.4
    4: [1,2,3],
    a: b # Variables a,b
}
```

#### Arithmethic operations

Negation
```
-1
```

Addition
```
1+2
```

Subtraction
```
3-1
```

Multiplication
```
1.1*98347
```

Division
```
42/24
```

Power
```
4.4^5
```

#### Concatenation

Only valid for strings
```
"a" + "baaa" # Equals "abaaa"
```

#### Logical operations

Negation
```
not true
```

And
```
true and true
```

Or
```
false or false
```

#### Comparisons

Less than
```
1 < 2
```

Less than or equal
```
1 <= 2
```

Greater than
```
2 > 1
```

Greater than or equal
```
2 >= 1
```

Equal
```
1 == 1
```

Not equal
```
1 != 2
```

#### Grouping

```
(2+3)*4

(1 > 2) != ((1 >= (3+4)^2) and (1 < 6))
```

#### Calling functions
```
foo("bar", bazz)
```

#### Accessing
```
list[0]
dict["ab"]
obj.prop
```

#### Slicing
Only valid for lists
```
list[1:]
list[1:7]
list[1::2]
list[::2]
list[1:7:2]
list[:7]
list[:7:2]
```

#### Assignment
```
a = 1
list[0] = 3
dict["b"] = "a"
obj.asd = 3
foo().bar.bazz[3]["b"] = 7
```

#### Anonymous functions
```
let square = fn (a) return a^2

let cube = fn (a) begin
    return a*a*a
end

let excellent = fn () "excellent"
```


## Statements

They are terminated by NEWLINE

#### Declarations

Variables
```
let a # Initialized as nil
let b = 1
```

Classes
```
class Buyable begin
    buy() begin
        println("buying " + this.description + ", for $" + this.price)
    end
end

class Item < Buyable begin
    # constructor
    init(price, description) begin
        this.price = price
        this.description = description
    end

    # instance method
    tellPrice() begin
        println(this.price)
    end

    # class method
    class sayHi() begin
        println("hi")
    end
end

# Usage

let item = Item("Coke", "1")

item.tellPrice() # prints: 1

item.buy() # prints: buying Coke for $1

Item.sayHi() # prints: hi
```

Functions

```
fn sum(a, b) begin
    return a+b
end

fn sum(a, b)
begin
    return a+b
end

fn sum(a, b) return a+b

fn sum(a, b) a+b
```


#### Expression

Expressions followed by NEWLINE

#### Block

```
let a = 2
begin
    let a = 1
    println(a * 2) # Prints 2
end
```

#### If-elif-else

```
if a > 2 begin
    println("a > 2")
end

if a > 2 begin
    println("a > 2")
elif a < 10
    println("2 < a < 10")
end

if a > 2 begin
    println("a > 2")
else
    println("a >= 10")
end

if a > 2 begin
    println("a > 2")
elif a < 10
    println("2 < a < 10")
else
    println("a >= 10")
end
```

#### While

```
while a > 2 begin
    a = a - 2
end

while a > 2 a = a - 2
```

#### Classic For
```
for let i = 0; i < 10; i = i+1 println(i)

for let i = 0; i < 10; i = i+1 begin
    println(i)
end
```

#### Enhanced For
```
let list = [1, 2, 3]

for i in list begin
    println(i)
end

for i in list println(i)
```

#### Return
Only valid inside functions
```
return expr

return
```
