var threw = false; try { Map.prototype.set.call({}, 'a', 1); } catch (e) { threw = e instanceof TypeError; } threw;
