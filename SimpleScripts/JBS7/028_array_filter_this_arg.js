var c = { min: 3 };
var b = [1, 3, 4].filter(function (x) { return x >= this.min; }, c);
b.length;
