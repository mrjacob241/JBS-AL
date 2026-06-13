function score(a) { var s = 0; for (var i = 0; i < a.length; i = i + 1) { if (a[i] > 2) { s = s + a[i]; } } return s; } var data = [1, 3, 5]; score(data);
