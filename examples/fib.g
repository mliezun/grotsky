fn fib(n) begin
    if n < 2 begin
        return n
    end
    return fib(n-2) + fib(n-1)
end
io.println(fib(30))