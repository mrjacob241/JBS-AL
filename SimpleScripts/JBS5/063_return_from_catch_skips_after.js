function f() { try { throw 9; } catch (e) { return e; } return 1; } f();
