var o = Object.create(null, { x: { value: 21, writable: false, enumerable: true, configurable: true } });
Object.getOwnPropertyDescriptor(o, "x").writable;
