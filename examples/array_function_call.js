// Array function call - tests calling functions stored in arrays
var funcs = [
  function() { return 1; },
  function() { return 2; },
  function() { return 3; }
];

console.log(funcs[0]()); // 1
console.log(funcs[1]()); // 2
console.log(funcs[2]()); // 3
