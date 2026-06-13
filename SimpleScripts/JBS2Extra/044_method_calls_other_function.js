function add(x) { return x + 2; } function m() { return add(this.x); } var o = { x: 5 }; o.m = m; o.m();
