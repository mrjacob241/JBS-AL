try { false || function() { throw 7; }(); } catch (e) { e; }
