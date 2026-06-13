var text = ''; var m = new Map([['a',1],['b',2]]); m.forEach(function (v,k) { text = text + k + v; }); text === 'a1b2';
