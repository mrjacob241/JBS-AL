function tri(a, b, c) { return a + b + c; } var f = tri.bind(undefined, 1).bind(undefined, 2); f(3);
