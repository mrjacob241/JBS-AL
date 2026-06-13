var threw = false; try { Map(); } catch (e) { threw = e instanceof TypeError; } threw;
