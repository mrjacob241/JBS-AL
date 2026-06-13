var o = Object();
Object.is(Object.defineProperty(o, "x", { value: 1, writable: true, enumerable: true, configurable: true }), o);
