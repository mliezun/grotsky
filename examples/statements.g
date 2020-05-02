# Variable definition
let asd = "asd"

# Block
begin
    let asd = "asd"
end

let a = 3
# If statement
if a > 2 begin
    a = a - 1
elif a < 3
    a = a + 1
else
    a = a * 2
end

# While
while a > 2 begin
    a = a + 1
end

# Classic for
for let i = 0, i < 10, i = i + 1 begin

end

# Enhanced for
for a, b in [[1, 2], [3, 4]] begin

end

# Function statement
fn foo(a) begin
    let b = [a, 1, 2, a^2*3]
end