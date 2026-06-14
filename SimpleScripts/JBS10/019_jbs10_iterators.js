var d = Object.getOwnPropertyDescriptor(Iterator.prototype, Symbol.toStringTag);
typeof d.get === 'function' && typeof d.set === 'function' && d.writable === undefined;
