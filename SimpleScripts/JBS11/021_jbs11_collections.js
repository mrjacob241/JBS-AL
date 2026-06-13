var threw = false; try { Set(); } catch (e) { threw = e instanceof TypeError; } threw;
