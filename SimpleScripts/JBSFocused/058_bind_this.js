function f(){return this.x;} var g=f.bind({x:9}); g();
