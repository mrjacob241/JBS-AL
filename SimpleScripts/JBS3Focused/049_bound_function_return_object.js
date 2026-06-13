function make(x) { return { v: x }; } var f = make.bind(undefined, 7); f().v;
