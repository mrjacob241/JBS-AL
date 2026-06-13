var x = 0; var f = function(v) { if (v !== 3) { throw new TypeError("bad"); } return v && 10; }; try { x = f(3); } catch (e) { x = 99; } x;
