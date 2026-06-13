# JBS Test262 Built-Ins Pipeline Report

- Root: `ECMAScript/test262-main/test/built-ins`
- Filter: `Array/prototype/flat`
- Limit: `none`
- Include unsupported: `false`
- Per-test timeout ms: `1000`
- Duration ms: `232`

## Counts

- Seen: `23585`
- Selected: `43`
- Passed: `10`
- Failed: `18`
- Timed out: `0`
- Unsupported skipped: `15`

## First Failures

- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/non-numeric-depth-should-not-throw.js`: `TypeError: numeric conversion failed`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/non-object-ctor-throws.js`: `TypeError: assert.throws expected a throw`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/null-undefined-elements.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/null-undefined-input-throws.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/proxy-access-count.js`: `ReferenceError: Proxy is not defined`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/target-array-non-extensible.js`: `TypeError: assert.throws expected a throw`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flat/target-array-with-non-configurable-property.js`: `TypeError: assert.throws expected a throw`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/array-like-objects-poisoned-length.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/array-like-objects-typedarrays.js`: `ReferenceError: Int32Array is not defined`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/array-like-objects.js`: `TypeError: cannot convert undefined or null to object`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/bound-function-argument.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/depth-always-one.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/non-callable-argument-throws.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/target-array-with-non-writable-property.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/this-value-ctor-object-species-custom-ctor-poisoned-throws.js`: `SyntaxError: unexpected token in expression: Punct('.')`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/this-value-ctor-object-species-custom-ctor.js`: `SyntaxError: unexpected token in expression: Punct('.')`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/this-value-null-undefined-throws.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/thisArg-argument.js`: `SyntaxError: expected ','`
