function Box(value) { this.value = value; }
Box.prototype.get = function () { return this.value; };
var b = new Box(9);
b.get();
