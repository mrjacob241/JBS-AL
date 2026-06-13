var o = {}; Object.defineProperty(o, "x", { value: 2, writable: true, enumerable: true, configurable: true }); verifyWritable(o, "x"); o.x = 5; o.x;
