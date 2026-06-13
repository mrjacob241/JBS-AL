var m = new Map(); m.set('a', 1); m.set('b', 2); var t = ''; for (var k of m.keys()) { t = t + k; } t === 'ab';
