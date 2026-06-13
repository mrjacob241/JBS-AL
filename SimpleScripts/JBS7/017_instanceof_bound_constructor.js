function Box() {}
var Bound = Box.bind(null);
var b = new Bound();
b instanceof Box;
