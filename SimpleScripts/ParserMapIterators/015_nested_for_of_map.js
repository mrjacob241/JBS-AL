var outer = new Map([["x", new Map([["y", 2]])]]); for (var entry of outer) { for (var inner of entry[1]) { inner[1]; } }
