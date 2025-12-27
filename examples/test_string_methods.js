// Test string methods
var s = "hello world";
console.log("Length:", s.length);
console.log("charAt(0):", s.charAt(0));
console.log("charCodeAt(0):", s.charCodeAt(0));
console.log("indexOf('world'):", s.indexOf("world"));
console.log("lastIndexOf('l'):", s.lastIndexOf("l"));
console.log("slice(0, 5):", s.slice(0, 5));
console.log("substring(0, 5):", s.substring(0, 5));
console.log("toLowerCase:", "HELLO".toLowerCase());
console.log("toUpperCase:", "hello".toUpperCase());

// Trim methods
console.log("trim:", "  hi  ".trim());
console.log("trimStart:", "  hi  ".trimStart());
console.log("trimEnd:", "  hi  ".trimEnd());

// Split
var parts = "a,b,c".split(",");
console.log("split:", parts[0], parts[1], parts[2]);
console.log("split empty:", "abc".split("").length);

// Replace
console.log("replace:", "hello".replace("l", "L"));
console.log("replaceAll:", "hello".replaceAll("l", "L"));

// Includes/startsWith/endsWith
console.log("includes:", "hello".includes("ell"));
console.log("startsWith:", "hello".startsWith("hel"));
console.log("endsWith:", "hello".endsWith("llo"));

// Concat
console.log("concat:", "a".concat("b", "c", 123));

// codePointAt
console.log("codePointAt(0):", "A".codePointAt(0));

// String.fromCharCode
console.log("fromCharCode:", String.fromCharCode(65, 66, 67));

// String.fromCodePoint
console.log("fromCodePoint:", String.fromCodePoint(65, 66, 67));
