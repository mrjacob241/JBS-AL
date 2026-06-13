function add(a, b) { return a + b; } var f = add.bind(undefined, 2); var a = [f(3)]; a.push(4); Object.values(a)[0] + a.pop();
