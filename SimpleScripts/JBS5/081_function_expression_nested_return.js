var outer = function(x) { var inner = function(y) { return y + 1; }; return inner(x) * 2; }; outer(4);
