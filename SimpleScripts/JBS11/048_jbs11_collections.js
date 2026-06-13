var s = new Set(); s.add('a'); s.add('b'); var t = ''; for (var v of s) { t = t + v; } t === 'ab';
