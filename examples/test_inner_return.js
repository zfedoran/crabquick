function outer() {
  return function() {
    console.log("inside inner");
    return 42;
  };
}
var f = outer();
console.log("about to call f");
f();
console.log("called f")
