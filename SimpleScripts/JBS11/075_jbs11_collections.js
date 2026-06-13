var text = ''; var s = new Set(['a']); s.forEach(function (v,k) { text = text + v + k; }); text === 'aa';
