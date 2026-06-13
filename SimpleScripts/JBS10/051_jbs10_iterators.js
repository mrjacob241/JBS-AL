var it = 'a'[Symbol.iterator](); Object.getPrototypeOf(Object.getPrototypeOf(it)) === Iterator.prototype;
