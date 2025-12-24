// Array operations example
var arr = [1, 2, 3, 4, 5];

console.log("Array length: " + arr.length);
console.log("First element: " + arr[0]);
console.log("Last element: " + arr[4]);

// Add elements
arr.push(6);
console.log("After push: length = " + arr.length);

// Remove element
var popped = arr.pop();
console.log("Popped: " + popped);
console.log("After pop: length = " + arr.length);

// Join array
var joined = arr.join(", ");
console.log("Joined: " + joined);
