var o = { a: 1 };
Object.defineProperty(o, "b", { value: 2, enumerable: false });
var count = 0;
for (var k in o) {
  count++;
}
count;
