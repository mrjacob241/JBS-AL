var seen = false; var m = new Map([['a',1]]); m.forEach(function (v,k,c) { seen = c === m; }); seen;
