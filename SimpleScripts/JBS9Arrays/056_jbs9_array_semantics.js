var a = [1,2]; Object.defineProperty(a, 'length', { writable: false }); a.length = 1; (1 in a);
