function get() { return this.x; } var f = get.bind({ x: 3 }); f.call({ x: 9 });
