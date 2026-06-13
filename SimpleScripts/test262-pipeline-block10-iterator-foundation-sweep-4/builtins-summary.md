# JBS Test262 Built-Ins Pipeline Report

- Root: `ECMAScript/test262-main/test/built-ins`
- Filter: `built-ins/Iterator`
- Limit: `none`
- Include unsupported: `false`
- Per-test timeout ms: `1000`
- Duration ms: `1138`

## Counts

- Seen: `23585`
- Selected: `514`
- Passed: `105`
- Failed: `84`
- Timed out: `0`
- Unsupported skipped: `325`

## First Failures

- `ECMAScript/test262-main/test/built-ins/Iterator/concat/iterable-primitive-wrapper-objects.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/return-is-forwarded.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/return-is-not-forwarded-after-exhaustion.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/return-is-not-forwarded-before-initial-start.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/return-method-called-with-zero-arguments.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/throws-typeerror-when-generator-is-running-next.js`: `thread 'main' (457579) has overflowed its stack fatal runtime error: stack overflow, aborting`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/throws-typeerror-when-generator-is-running-return.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/throws-typeerror-when-iterable-not-an-object.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/throws-typeerror-when-iterator-method-not-callable.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Iterator/concat/throws-typeerror-when-iterator-not-an-object.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Iterator/from/get-return-method-when-call-return.js`: `ReferenceError: TemporalHelpers is not defined`
- `ECMAScript/test262-main/test/built-ins/Iterator/from/primitives.js`: `SyntaxError: expected ','`
- `ECMAScript/test262-main/test/built-ins/Iterator/from/return-method-calls-base-return-method.js`: `ReferenceError: TemporalHelpers is not defined`
- `ECMAScript/test262-main/test/built-ins/Iterator/from/return-method-returns-iterator-result.js`: `TypeError: value is not iterable`
- `ECMAScript/test262-main/test/built-ins/Iterator/from/return-method-throws-for-invalid-this.js`: `TypeError: value is not iterable`
- `ECMAScript/test262-main/test/built-ins/Iterator/proto-from-ctor-realm.js`: `ReferenceError: $262 is not defined`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/invokes-return.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/is-function.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/length.js`: `TypeError: cannot convert undefined or null to object`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/name.js`: `TypeError: cannot convert undefined or null to object`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/prop-desc.js`: `TypeError: verifyProperty target property is missing`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.dispose/return-val.js`: `TypeError: value is not callable`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/Symbol.toStringTag/prop-desc.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/constructor/prop-desc.js`: `TypeError: assert.sameValue failed`
- `ECMAScript/test262-main/test/built-ins/Iterator/prototype/drop/argument-effect-order.js`: `TypeError: assert.throws expected a throw`
