var o = Object();
Object.defineProperty(o, true, { value: "bool", writable: true, enumerable: true, configurable: true });
Object.getOwnPropertyDescriptor(o, "true").value;
