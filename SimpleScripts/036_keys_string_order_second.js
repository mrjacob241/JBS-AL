var o = { b: 1, 0: "zero", a: 2 };
Object.getOwnPropertyDescriptor(Object.keys(o), "1").value;
