var o = { a: 1, b: 2 };
delete o.a;
var count = 0;
for (var k in o) {
  count++;
}
count;
