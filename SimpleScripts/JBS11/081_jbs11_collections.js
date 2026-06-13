var next = Object.getPrototypeOf(new Map().keys()).next; var threw = false; try { next.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
