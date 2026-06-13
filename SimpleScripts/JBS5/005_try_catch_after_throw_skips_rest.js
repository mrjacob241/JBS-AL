var x = 1; try { x = 2; throw 0; x = 9; } catch (e) { x = x + 1; } x;
