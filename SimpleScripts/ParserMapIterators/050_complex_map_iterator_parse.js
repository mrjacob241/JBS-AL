function sumValues(map) { var s = 0; for (var entry of map.entries()) { if (entry[1] > 1) { s = s + entry[1]; } } return s; } sumValues(new Map([["a", 1], ["b", 2], ["c", 3]]));
