fn fib(n) begin
    if n < 2 return n
    return fib(n-2) + fib(n-1)
end
println(fib(30))