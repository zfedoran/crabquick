// Test Object methods

// Object.create()
var proto = { greet: function() { return "hello"; } };
var obj = Object.create(proto);
console.log("Object.create - has prototype:", obj.greet !== undefined);

// Object.getPrototypeOf()
var p = Object.getPrototypeOf(obj);
console.log("getPrototypeOf:", p === proto);

// Object.setPrototypeOf()
var newProto = { name: "new" };
var o = {};
Object.setPrototypeOf(o, newProto);
console.log("setPrototypeOf works:", Object.getPrototypeOf(o) === newProto);

// Object.defineProperty()
var x = {};
Object.defineProperty(x, "value", {
    value: 42,
    writable: true,
    enumerable: true,
    configurable: true
});
console.log("defineProperty value:", x.value);

// Object.prototype.hasOwnProperty()
var h = { a: 1 };
console.log("hasOwnProperty a:", h.hasOwnProperty("a"));
console.log("hasOwnProperty b:", h.hasOwnProperty("b"));

// Object.prototype.toString() - direct call (Function.call not yet implemented)
var o = {};
console.log("toString object:", o.toString());

// Object.keys()
var k = { a: 1, b: 2, c: 3 };
var keys = Object.keys(k);
console.log("Object.keys length:", keys.length);

// Object.values()
var vals = Object.values(k);
console.log("Object.values length:", vals.length);

// Object.entries()
var entries = Object.entries(k);
console.log("Object.entries length:", entries.length);

// Object.assign()
var target = { a: 1 };
var source = { b: 2 };
Object.assign(target, source);
console.log("Object.assign - target.a:", target.a);
console.log("Object.assign - target.b:", target.b);
