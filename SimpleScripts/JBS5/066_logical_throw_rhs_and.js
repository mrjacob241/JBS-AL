try { true && function() { throw 6; }(); } catch (e) { e; }
