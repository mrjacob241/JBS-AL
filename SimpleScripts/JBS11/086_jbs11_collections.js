var threw = false; try { Map.prototype.delete.call({}, 'a'); } catch (e) { threw = e instanceof TypeError; } threw;
