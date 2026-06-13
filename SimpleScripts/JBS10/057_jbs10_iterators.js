var it = 'ab'[Symbol.iterator](); it.next(); it.next(); it.next().done === true;
