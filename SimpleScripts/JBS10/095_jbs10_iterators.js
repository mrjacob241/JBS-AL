var p = Object.getPrototypeOf([].values()); p.next.call([9].values()).value === 9;
