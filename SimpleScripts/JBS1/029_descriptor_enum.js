var o = Object(); Object.defineProperty(o, "x", { value: 9, writable: false, enumerable: true, configurable: true }); Object.getOwnPropertyDescriptor(o, "x").enumerable;
