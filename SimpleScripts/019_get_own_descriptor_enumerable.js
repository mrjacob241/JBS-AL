var o = Object();
Object.defineProperty(o, "x", { value: 1, writable: false, enumerable: true, configurable: false });
Object.getOwnPropertyDescriptor(o, "x").enumerable;
