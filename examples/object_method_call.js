// Object method call - tests calling functions stored in object properties
var obj = {
  greet: function() { return "hello"; },
  add: function(a, b) { return a + b; }
};

console.log(obj.greet()); // hello
console.log(obj.add(2, 3)); // 5
