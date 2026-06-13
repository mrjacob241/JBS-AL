var next = Object.getPrototypeOf(''[Symbol.iterator]()).next; var threw = false; try { next.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
