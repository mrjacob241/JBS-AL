function set(v) { this.x = v; return this.x; } var o = { x: 0 }; o.set = set; o.set(6) + o.x;
