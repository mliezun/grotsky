class HelloWorld begin
    hello() begin
        return {"body": "<!DOCTYPE html><html><head><title>Grotsky HTTP Server</title></head><body><h1>Grotsky HTTP Server</h1><div>Hello, world</div></body></html>"}
    end
end

class ByeBye < HelloWorld begin
    bye() begin
        return {"body": "<!DOCTYPE html><html><head><title>Grotsky HTTP Server</title></head><body><h1>Grotsky HTTP Server</h1><div>bye bye birdie</div></body></html>"}
    end

    hello() begin
        io.println("overwriting hello")
        return super.hello()
    end
end

let h = ByeBye()

http.handler("/", h.hello)
http.handler("/bye", h.bye)

http.listen(":8092")
