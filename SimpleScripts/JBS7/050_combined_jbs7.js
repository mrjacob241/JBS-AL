function Box(value) { this.value = value; }
var b = new Box(4);
var total = 0;
for (var k in { a: 1, b: 2 }) { total = total + 1; }
var mapped = [1, 2, 3].map(function (x) { return x + b.value; });
mapped.reduce(function (acc, x) { return acc + x; }, 0) + total + (b instanceof Box ? 0 : 100);
