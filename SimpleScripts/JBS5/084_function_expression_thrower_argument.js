function run(fn) { return fn(); } try { run(function() { throw 14; }); } catch (e) { e; }
