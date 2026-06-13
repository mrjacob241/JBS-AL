function f() { throw new TypeError("nested"); } assert.throws(TypeError, function() { f(); }); true;
