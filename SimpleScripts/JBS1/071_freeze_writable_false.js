var o = { x: 1 }; Object.freeze(o); Object.getOwnPropertyDescriptor(o, "x").writable;
