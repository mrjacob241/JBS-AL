var acc = [1, 2, 3].reduce(function (box, x) { box.total = box.total + x; return box; }, { total: 0 });
acc.total;
