var threw = false; try { Iterator(); } catch (e) { threw = e instanceof TypeError; } threw;
