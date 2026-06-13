var s = new Set(); s.add('a'); s.add('b'); var t = ''; for (var p of s.entries()) { t = t + p[0] + p[1]; } t === 'aabb';
