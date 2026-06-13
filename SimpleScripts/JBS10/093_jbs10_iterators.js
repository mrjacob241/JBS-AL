var next = Object.getPrototypeOf([].values()).next; var threw = false; try { next.call(undefined); } catch (e) { threw = e instanceof TypeError; } threw;
