var o = {}; Object.defineProperty(o, "x", { value: 1, writable: true, enumerable: true, configurable: true }); verifyEnumerable(o, "x"); o.propertyIsEnumerable("x");
