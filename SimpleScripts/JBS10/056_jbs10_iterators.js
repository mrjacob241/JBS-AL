var it = 'ab'[Symbol.iterator](); it.next().value === 'a' && it.next().value === 'b';
