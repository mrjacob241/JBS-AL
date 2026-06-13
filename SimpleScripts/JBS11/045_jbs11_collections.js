var s = new Set(); s.add('a'); s.add('b'); var t = ''; for (var v of s.values()) { t = t + v; } t === 'ab';
