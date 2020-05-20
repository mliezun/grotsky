fn main(a) begin

    if a < 100 begin
        a = a * 2
        io.println("a < 100", a)
    end

    while a < 100 and 1 == 1 begin
        a = a*(2+3)^0.9
    end

    if a == 100 begin
        io.println("a = 100")
    elif a >= 1000
        io.println("a > 1000")
    else
        io.println("100 < a < 1000")
    end

    return a
end

for let i = 0; i < 10; i = i+1 begin
    io.println(main((i+1)^i))
end

for i in [1, 2, 3] begin
    io.println(i)
end

for key, val in {"a": "b", "c": "d"} begin
    io.println(key)
    io.println(val)
end

fn map(array, mapFn) begin
    for i in array begin
        io.println(mapFn(i))
    end
end

map([1, 2, 3], fn(i) i^3)

fn foo(a) a*2
io.println(foo(1))
