var next = Object.getPrototypeOf([].values()).next; var threw = false; try { next.call({}); } catch (e) { threw = e instanceof TypeError; } threw;
