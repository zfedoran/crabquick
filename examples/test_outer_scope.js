function outer() {
  console.log("in outer");
  function inner() {
    console.log("in inner");
    return 123;
  }
  console.log("after inner def");
  var x = inner();
  console.log("x is:");
  console.log(x);
  return x;
}
console.log("calling outer");
console.log(outer())
