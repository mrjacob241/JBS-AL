var o = Object(); Object.defineProperty(o, "x", { value: 9, writable: false, enumerable: true, configurable: false }); Object.getOwnPropertyDescriptor(o, "x").configurable;
