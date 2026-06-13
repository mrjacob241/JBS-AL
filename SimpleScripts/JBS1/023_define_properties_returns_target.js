var o = Object(); Object.is(Object.defineProperties(o, { x: { value: 1, writable: true, enumerable: true, configurable: true } }), o);
