function score(a) { var s = 0; for (var i = 0; i < a.length; i = i + 1) { if (a[i].v > 2) { s = s + a[i].v; } } return s; } score([{ v: 1 }, { v: 3 }, { v: 5 }]);
