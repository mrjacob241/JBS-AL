var box = { x: 0 }; function bump() { box.x = box.x + 1; return true; } true || bump(); box.x;
