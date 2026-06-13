var box = { x: 0 }; function bump() { box.x = box.x + 1; return true; } false && bump(); box.x;
