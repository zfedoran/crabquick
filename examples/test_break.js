console.log("start");
var arr = [1, 2, 3, 4, 5];
var sum = 0;
for (var x of arr) {
    console.log("x = " + x);
    if (x > 3) {
        console.log("breaking");
        break;
    }
    sum = sum + x;
}
console.log("sum = " + sum);
