function C(len) { this.len = len; } var obj = Array.of.call(C, 'a', 'b'); new AggregateError(obj, 'many').errors.length;
