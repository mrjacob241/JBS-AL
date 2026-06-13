function f(a, b) { return a * b; } var g = f.bind(undefined, 3); var h = g.bind(undefined, 4); h();
