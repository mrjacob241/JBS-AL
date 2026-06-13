var o = { x: 13 }; try { try { throw o; } catch (e) { throw e; } } catch (e) { e.x; }
