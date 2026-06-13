var p = { x: 11 };
var o = Object();
Object.setPrototypeOf(o, p);
o.x;
