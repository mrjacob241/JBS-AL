var o = { x: 9 }; function f() { return this.x; } o.f = f; o.f();
