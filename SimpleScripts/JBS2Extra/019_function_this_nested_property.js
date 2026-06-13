function get() { return this.a.b; } var o = { a: { b: 11 } }; o.get = get; o.get();
