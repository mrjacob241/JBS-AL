var a = [1]; var it = a[Symbol.iterator](); a[1] = 2; it.next(); it.next().value;
