try { try { throw 1; } catch (e) { throw e + 6; } } catch (e) { e + 1; }
