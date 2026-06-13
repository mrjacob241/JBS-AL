var s = Object(); Object.defineProperty(s, "x", { value: 1, writable: true, enumerable: false, configurable: true }); var t = Object(); Object.assign(t, s); Object.hasOwn(t, "x");
