function Box(value) { this.value = value; }
var Bound = Box.bind(null, 3);
var b = new Bound();
b instanceof Box;
