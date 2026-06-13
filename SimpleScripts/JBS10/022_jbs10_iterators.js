var threw = false; try { new Iterator(); } catch (e) { threw = e instanceof TypeError; } threw;
