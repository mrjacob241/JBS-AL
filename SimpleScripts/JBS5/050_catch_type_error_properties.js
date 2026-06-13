try { throw new TypeError("bad"); } catch (e) { e.name + ":" + e.message; }
