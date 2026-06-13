var p = { x: 1 };
var o = Object.create(p);
Object.is(Object.getPrototypeOf(Object.preventExtensions(o)), p);
