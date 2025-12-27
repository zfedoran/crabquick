var funcs = [function() { return 1; }];
console.log("funcs:", funcs);
console.log("funcs[0]:", funcs[0]);
console.log("typeof funcs[0]:", typeof funcs[0]);
var f = funcs[0];
console.log("f:", f);
console.log("typeof f:", typeof f);
console.log("calling f():", f());
