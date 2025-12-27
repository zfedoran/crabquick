function add(a, b) {
    return a + b;
}
var obj = {
    adder: add
};
console.log("Result: " + obj.adder(2, 3));
