var o = { a: 1, b: 2, c: 3 };
var count = 0;
for (var k in o) { count = count + 1; delete o.c; }
count;
