var self = {x: 5}; var seen = 0; var s = new Set(['a']); s.forEach(function () { seen = this.x; }, self); seen === 5;
