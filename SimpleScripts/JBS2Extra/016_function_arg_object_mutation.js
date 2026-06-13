function f(o) { o.a = o.a + 4; return o.a; } var o = { a: 1 }; f(o) + o.a;
