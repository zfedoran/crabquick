function makeAdder(x) {
  console.log("makeAdder called with:");
  console.log(x);
  return function(y) {
    console.log("inner called with y=");
    console.log(y);
    console.log("x is:");
    console.log(x);
    return x + y;
  };
}

var add5 = makeAdder(5);
console.log("add5 result:");
console.log(add5(3));

console.log("---");

var add10 = makeAdder(10);
console.log("add10 result:");
console.log(add10(3));
