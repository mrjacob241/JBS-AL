var threw = false; try { Set.prototype.add.call({}, 'a'); } catch (e) { threw = e instanceof TypeError; } threw;
