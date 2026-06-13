var c = { n: 3 };
var b = [1, 2].map(function (x) { return x + this.n; }, c);
b[0] + b[1];
