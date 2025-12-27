// Test additional array methods

// lastIndexOf
var arr = [10, 11, 10, 11];
console.log("lastIndexOf(11):", arr.lastIndexOf(11));
console.log("lastIndexOf(11, 2):", arr.lastIndexOf(11, 2));
console.log("lastIndexOf(9):", arr.lastIndexOf(9));

// reduceRight
var nums = [1, 2, 3, 4];
var result = nums.reduceRight(function(acc, val) { return acc + val; }, "");
console.log("reduceRight string:", result);

var sum = nums.reduceRight(function(acc, val) { return acc + val; }, 0);
console.log("reduceRight sum:", sum);

// sort (default - lexicographic)
var a = [5, 4, 3, 2, 1];
a.sort();
console.log("sort default:", a[0], a[1], a[2], a[3], a[4]);

// sort with comparator (numeric descending)
var b = [1, 2, 3, 4, 5];
b.sort(function(x, y) { return y - x; });
console.log("sort desc:", b[0], b[1], b[2], b[3], b[4]);

// sort with comparator (numeric ascending)
var c = [5, 1, 4, 2, 3];
c.sort(function(x, y) { return x - y; });
console.log("sort asc:", c[0], c[1], c[2], c[3], c[4]);

// toString
var d = [1, 2, 3];
console.log("toString:", d.toString());
