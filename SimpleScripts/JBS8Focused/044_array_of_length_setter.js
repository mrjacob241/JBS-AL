function C() { Object.defineProperty(this, 'length', { set: function (v) { this.seen = v; }, configurable: true }); } Array.of.call(C, 1, 2, 3, 4).seen;
