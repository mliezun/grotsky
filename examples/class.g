class HelloWorld begin
    init(name) begin
        this.name = name
    end

    hello() begin
        return "Hello, " + this.name
    end
end

class ByeBye < HelloWorld begin
    init(name, msg) begin
        super(name)
        this.msg = msg
    end

    bye() begin
        io.println("Bye bye", this.msg)
    end

    hello() begin
        io.println(super.hello())
    end
end

let b = ByeBye("Grotsky", "42")

b.hello()
b.bye()
