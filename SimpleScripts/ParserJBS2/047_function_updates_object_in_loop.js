function bump(o) { o.x = o.x + 1; return o.x; } var o = { x: 0 }; for (var i = 0; i < 3; i = i + 1) { bump(o); } o.x;
