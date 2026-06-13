function get() { return this.x; } var f = get.bind({ x: 7 }); f();
