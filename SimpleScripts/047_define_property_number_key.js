var o = Object();
Object.defineProperty(o, 7, { value: "num", writable: true, enumerable: true, configurable: true });
Object.getOwnPropertyDescriptor(o, "7").value;
