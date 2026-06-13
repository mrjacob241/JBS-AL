var o = {}; o[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: 'v' + i, done: i > 2 }; } }; }; Array.from(o).length;
