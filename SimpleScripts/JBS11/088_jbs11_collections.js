var threw = false; try { Set.prototype.has.call({}, 'a'); } catch (e) { threw = e instanceof TypeError; } threw;
