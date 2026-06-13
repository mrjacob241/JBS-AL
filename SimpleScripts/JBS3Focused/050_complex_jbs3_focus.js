function add(a, b) { return a + b; } var f = add.bind(undefined, 2); var a = []; Array.prototype.push.call(a, f(5)); a[1] = 3; a.length + a[0] + a.pop();
