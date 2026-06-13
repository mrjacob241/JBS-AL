var p = { x: 1 };
var o = Object.create(p);
o.x = 2;
var count = 0;
for (var k in o) {
  count++;
}
count;
