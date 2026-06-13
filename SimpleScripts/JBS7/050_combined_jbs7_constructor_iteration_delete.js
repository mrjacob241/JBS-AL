function Box(value) {
  this.value = value;
  this.extra = 3;
}
var b = new Box(4);
delete b.extra;
var total = 0;
for (var k in b) {
  if (k in b) {
    total = total + b[k];
  }
}
total + (b instanceof Box ? 8 : 0);
