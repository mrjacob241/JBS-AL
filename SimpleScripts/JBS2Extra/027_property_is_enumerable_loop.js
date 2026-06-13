var o = { a: 1 }; var s = 0; for (var i = 0; i < 3; i = i + 1) { if (Object.prototype.propertyIsEnumerable.call(o, "a")) { s = s + 1; } } s;
