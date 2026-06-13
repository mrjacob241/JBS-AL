var o = { length: 2, 0: 'a', 1: 'b' }; Array.prototype.pop.call(o); (1 in o);
