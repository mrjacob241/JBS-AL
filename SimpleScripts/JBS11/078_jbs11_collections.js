var seen = false; var s = new Set(['a']); s.forEach(function (v,k,c) { seen = c === s; }); seen;
