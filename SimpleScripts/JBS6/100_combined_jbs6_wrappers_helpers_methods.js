var key = new String("n"); var o = { [key]: new Number("40"), add(v) { return this.n.valueOf() + Number(v); } }; verifyEqualTo(o, "n", o.n); o.add(new String("2"));
