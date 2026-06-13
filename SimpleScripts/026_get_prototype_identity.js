var p = Object();
var o = Object.create(p);
Object.is(Object.getPrototypeOf(o), p);
