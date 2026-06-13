var threw = false; try { Set.prototype.clear.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
