var d = { value: 77, writable: true, enumerable: true, configurable: true };
var o = Object();
Object.defineProperty(o, "x", d);
o.x;
