var o = { a: 1, b: 2 }; var s = 0; for (var i = 0; i < 2; i = i + 1) { if (Object.hasOwn(o, "a")) { s = s + 1; } } s;
