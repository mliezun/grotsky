fn fib(n) begin
    if n < 2 begin
        return n
    end
    return fib(n-1) + fib(n-2)
end
println(fib(30))