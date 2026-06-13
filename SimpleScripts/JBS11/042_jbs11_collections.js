var m = new Map(); m.set('a', 1); m.set('b', 2); var t = ''; for (var v of m.values()) { t = t + v; } t === '12';
