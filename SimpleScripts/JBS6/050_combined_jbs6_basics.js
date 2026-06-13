var k = "score"; var o = { [k]: new Number(10), read() { return this.score.valueOf() + Number("2"); } }; verifyEqualTo(o, "score", o.score); o.read();
