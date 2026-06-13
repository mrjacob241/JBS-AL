try { try { throw 3; } catch (e) { throw e * 4; } } catch (e) { e; }
