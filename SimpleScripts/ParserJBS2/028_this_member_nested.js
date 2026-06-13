function f() { return this.a.b; } var o = { a: { b: 8 } }; o.f = f; o.f();
