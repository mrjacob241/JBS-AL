var o = Object();
Object.defineProperty(o, "hidden", { value: 1, writable: true, enumerable: false, configurable: true });
Object.getOwnPropertyNames(o)[0];
