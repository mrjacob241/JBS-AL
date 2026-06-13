var target = { x: 1 };
Object.assign(target, null, undefined);
target.x;
