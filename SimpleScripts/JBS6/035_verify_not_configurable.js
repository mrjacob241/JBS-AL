var o = {}; Object.defineProperty(o, "x", { value: 1, writable: true, enumerable: true, configurable: false }); verifyNotConfigurable(o, "x"); true;
