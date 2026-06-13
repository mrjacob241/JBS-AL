var m = new Map(); m.set('a', 1); m.set('b', 2); m.delete('a'); m.set('a', 3); var t = ''; for (var p of m) { t = t + p[0] + p[1]; } t === 'b2a3';
