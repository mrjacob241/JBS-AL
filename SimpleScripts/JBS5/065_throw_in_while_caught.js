var i = 0; try { while (i < 3) { i = i + 1; if (i === 2) { throw i; } } } catch (e) { i = e + 5; } i;
