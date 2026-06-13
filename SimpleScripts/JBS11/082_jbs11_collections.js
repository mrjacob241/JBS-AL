var next = Object.getPrototypeOf(new Set().values()).next; var threw = false; try { next.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
