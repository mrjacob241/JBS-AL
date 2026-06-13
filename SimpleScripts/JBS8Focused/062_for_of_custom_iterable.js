var o = {}; o[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i, done: i > 3 }; } }; }; var sum = 0; for (var x of o) { sum = sum + x; } sum;
