var input = ['x']; var e = new AggregateError(input, 'many'); e.errors !== input && e.errors[0] === 'x';
