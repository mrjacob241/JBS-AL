var o = {}; o[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i, done: i > 2 }; } }; }; new AggregateError(o, 'many').errors.length;
