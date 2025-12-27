// Basic arrow function with expression body
var double = x => x * 2;
console.log("double(5):", double(5));

// Arrow with multiple params
var add = (a, b) => a + b;
console.log("add(3, 4):", add(3, 4));

// Arrow with no params
var five = () => 5;
console.log("five():", five());

// Arrow with block body
var greet = name => {
    var msg = "Hello, " + name;
    return msg;
};
console.log(greet("World"));

// Arrow with closure
function makeAdder(x) {
    return y => x + y;
}
var add10 = makeAdder(10);
console.log("add10(5):", add10(5));

// Arrow in array methods
var nums = [1, 2, 3, 4, 5];
var doubled = nums.map(x => x * 2);
console.log("Doubled:", doubled[0], doubled[1], doubled[2], doubled[3], doubled[4]);
