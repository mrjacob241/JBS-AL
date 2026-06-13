var self = {}; Array.from([1], function () { return this === self; }, self)[0] === true;
