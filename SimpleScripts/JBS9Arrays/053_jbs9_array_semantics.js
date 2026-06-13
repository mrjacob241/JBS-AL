var a = [1,2]; Object.defineProperty(a, 'length', { writable: false }); a[2] = 3; a.length;
