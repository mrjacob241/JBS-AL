var self = {x: 4}; var seen = 0; var m = new Map([['a',1]]); m.forEach(function () { seen = this.x; }, self); seen === 4;
