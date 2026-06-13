try { var a = []; a.length = 4294967296; 'bad'; } catch (e) { e.name; }
