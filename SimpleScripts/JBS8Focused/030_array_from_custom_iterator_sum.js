var o = {}; o[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i, done: i > 3 }; } }; }; var a = Array.from(o); a[0] + a[1] + a[2];
