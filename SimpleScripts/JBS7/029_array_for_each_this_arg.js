var c = { n: 3 };
var sum = 0;
[1, 2].forEach(function (x) { sum = sum + x + this.n; }, c);
sum;
