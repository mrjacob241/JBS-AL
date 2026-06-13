var obj = {}; obj[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i * 2, done: i > 3 }; } }; }; Array.from(obj)[2];
