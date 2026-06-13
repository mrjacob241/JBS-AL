var threw = false; try { Set.prototype.forEach.call(new Set(), 1); } catch (e) { threw = e instanceof TypeError; } threw;
