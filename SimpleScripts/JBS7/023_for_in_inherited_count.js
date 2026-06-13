var p = { a: 1 };
var o = Object.create(p);
o.b = 2;
var count = 0;
for (var k in o) {
  count++;
}
count;
