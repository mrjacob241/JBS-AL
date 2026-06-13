var value = 11; var o = { value }; verifyProperty(o, "value", { value, writable: true, enumerable: true, configurable: true }); o.value;
