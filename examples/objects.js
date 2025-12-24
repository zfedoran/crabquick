// Object operations example
var person = {
    name: "Alice",
    age: 30,
    city: "New York"
};

console.log("Name: " + person.name);
console.log("Age: " + person.age);

// Add new property
person.country = "USA";
console.log("Country: " + person.country);

// Modify property
person.age = 31;
console.log("New age: " + person.age);

// Object with method
var calculator = {
    add: function(a, b) {
        return a + b;
    },
    multiply: function(a, b) {
        return a * b;
    }
};

console.log("2 + 3 = " + calculator.add(2, 3));
console.log("4 * 5 = " + calculator.multiply(4, 5));
