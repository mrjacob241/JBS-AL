var threw = false; try { Map.prototype.get.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
