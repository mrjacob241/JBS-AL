var s = new Set(); s.add('a'); s.add('b'); s.delete('a'); s.add('a'); var t = ''; for (var v of s) { t = t + v; } t === 'ba';
