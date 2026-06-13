var o = {}; Object.defineProperty(o, "x", { value: 1, writable: true, enumerable: false, configurable: true }); verifyNotEnumerable(o, "x"); o.propertyIsEnumerable("x");
