var o = Object.create(null, { x: { value: 8, writable: false, enumerable: true, configurable: true } }); Object.getOwnPropertyDescriptor(o, "x").writable;
