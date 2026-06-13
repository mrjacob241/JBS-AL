var a = [1,2]; Object.defineProperty(a, 'length', { writable: false }); try { Object.defineProperty(a, '2', { value: 3 }); 'bad'; } catch (e) { e.name; }
