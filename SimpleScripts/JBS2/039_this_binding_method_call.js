var o = { x: 7 }; function get(){ return this.x; } o.get = get; o.get();
