var o = { x: 1 }; Object.seal(o); Object.getOwnPropertyDescriptor(o, "x").configurable;
