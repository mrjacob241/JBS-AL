var o = Object();
Object.defineProperty(o, "x", { value: 1, writable: true, enumerable: true, configurable: true });
Object.hasOwn(o, "x");
