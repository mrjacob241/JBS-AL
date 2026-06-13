function Box() {}
Box.prototype.flag = 1;
var b = new Box();
b.flag === 1 && b instanceof Box;
