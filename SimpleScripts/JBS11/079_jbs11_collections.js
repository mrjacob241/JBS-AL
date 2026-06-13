var threw = false; try { Map.prototype.forEach.call(new Map(), 1); } catch (e) { threw = e instanceof TypeError; } threw;
