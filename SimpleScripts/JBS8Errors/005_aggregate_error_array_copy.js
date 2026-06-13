var input = [1, 2]; var e = new AggregateError(input, "many"); e.errors !== input;
