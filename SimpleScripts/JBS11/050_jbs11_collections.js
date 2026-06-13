var s = new Set(); s.add('a'); var it = s.values(); it[Symbol.iterator]() === it;
