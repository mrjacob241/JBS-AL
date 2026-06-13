var p = Object.getPrototypeOf(''[Symbol.iterator]()); p.next.call('z'[Symbol.iterator]()).value === 'z';
