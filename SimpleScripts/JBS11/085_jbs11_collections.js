var threw = false; try { Map.prototype.has.call({}, 'a'); } catch (e) { threw = e instanceof TypeError; } threw;
