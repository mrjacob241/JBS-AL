var o = {}; Object.defineProperty(o, "x", { value: 3, writable: false, enumerable: true, configurable: true }); verifyNotWritable(o, "x"); try { o.x = 9; } catch (e) { } o.x;
