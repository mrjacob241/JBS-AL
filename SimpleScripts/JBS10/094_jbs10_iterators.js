var next = Object.getPrototypeOf([].values()).next; var threw = false; try { next.call(null); } catch (e) { threw = e instanceof TypeError; } threw;
