function C() { Object.defineProperty(this, 'length', { set: function (v) { this.seen = v; }, configurable: true }); } Array.from.call(C, {0: 'x', 1: 'y', length: 2}).seen;
