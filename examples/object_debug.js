var obj = {
  greet: function() { return "hello"; }
};
console.log("obj:", obj);
console.log("obj.greet:", obj.greet);
console.log("typeof obj.greet:", typeof obj.greet);
var f = obj.greet;
console.log("f:", f);
console.log("typeof f:", typeof f);
console.log("f():", f());
