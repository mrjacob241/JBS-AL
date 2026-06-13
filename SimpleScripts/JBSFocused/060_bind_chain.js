function f(a,b,c){return a+b+c;} var g=f.bind(undefined,1).bind(undefined,2); g(3);
