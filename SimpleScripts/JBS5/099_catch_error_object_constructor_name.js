try { throw Error("x"); } catch (e) { e.constructor.name; }
