var o = Object();
Object.defineProperty(o, "x", { value: "alpha", writable: false, enumerable: true, configurable: false });
Object.getOwnPropertyDescriptor(o, "x").value;
