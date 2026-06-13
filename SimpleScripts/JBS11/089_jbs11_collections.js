var threw = false; try { Set.prototype.delete.call({}, 'a'); } catch (e) { threw = e instanceof TypeError; } threw;
