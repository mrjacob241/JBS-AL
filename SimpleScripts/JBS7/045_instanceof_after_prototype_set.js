function Box() {}
var p = {};
Box.prototype = p;
var b = new Box();
b instanceof Box;
