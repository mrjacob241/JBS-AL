function add(a,b){return a+b;} var f=add.bind(undefined,2); var a=[]; a.push(f(3)); Object.defineProperty(a,"x",{value:4,enumerable:true}); Object.values(a)[0]+Object.values(a)[1];
