function add(a, b) { return a + b; } add.apply(undefined, { 0: 4, 1: 6, length: 2 });
