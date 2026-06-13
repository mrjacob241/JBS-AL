var vals = Object.values({ a: 2, b: 4, c: 6 }); var s = 0; for (var i = 0; i < vals.length; i = i + 1) { if (vals[i] > 3) { s = s + vals[i]; } } s;
