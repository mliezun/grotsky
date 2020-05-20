class HelloWorld begin
    fn hello() begin
        return {"body": "<!DOCTYPE html><html><head><title>Grotsky HTTP Server</title></head><body><h1>Grotsky HTTP Server</h1><div>Hello, world</div></body></html>"}
    end

    fn bye() begin
        return {"body": "<!DOCTYPE html><html><head><title>Grotsky HTTP Server</title></head><body><h1>Grotsky HTTP Server</h1><div>bye bye birdie</div></body></html>"}
    end
end

let h = HelloWorld()

http.handler("/", h.hello)
http.handler("/bye", h.bye)

http.listen(":8092")
