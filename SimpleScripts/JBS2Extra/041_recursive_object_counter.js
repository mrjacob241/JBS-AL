function f(n, o) { if (n <= 0) { return o.x; } o.x = o.x + 1; return f(n - 1, o); } var o = { x: 0 }; f(4, o) + o.x;
