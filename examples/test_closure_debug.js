function outer() {
  var x = 10;
  console.log("inside outer, x=");
  console.log(x);
  return function() {
    console.log("inside inner");
    return x;
  };
}
console.log("before outer");
var f = outer();
console.log("f =");
console.log(f);
console.log("calling f");
var r = f();
console.log("result =");
console.log(r);
