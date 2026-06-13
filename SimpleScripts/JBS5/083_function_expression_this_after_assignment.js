var o = { x: 10 }; var f = function() { return this.x + 2; }; o.f = f; o.f();
