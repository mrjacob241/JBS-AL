try { try { throw 4; } catch (e) { throw e + 4; } } catch (e) { e; }
