var m = new Map(); m.set('a', 1); var it = m.keys(); it[Symbol.iterator]() === it;
