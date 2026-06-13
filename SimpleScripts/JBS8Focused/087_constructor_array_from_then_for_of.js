function C(len) { this.len = len; } var obj = Array.from.call(C, {0: 1, 1: 2, length: 2}); var sum = obj.len; for (var x of [obj[0], obj[1]]) { sum = sum + x; } sum;
