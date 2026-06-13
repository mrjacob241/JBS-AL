var it = 'a'[Symbol.iterator](); it.next(); it.next().done === true && it.next().done === true;
