# JBS-7-Close Built-Ins Failures

This is a read-only triage list for failures in the current
`SimpleScripts/test262-pipeline/builtins-results.jsonl` report that look close to
JBS-7 rather than clearly future-stage.

## Representative Rows

- `Array/isArray/15.4.3.2-0-5.js`
  - Expected: `Array.isArray(Array.prototype) === true`.
  - Current signal: `Array.prototype` is installed as an ordinary object, not an
    array exotic object.

- `Object/assign/Override-notstringtarget.js`
  - Expected: `Object.assign(12, "aaa", "bb2b", "1c")` boxes target and string
    sources correctly.
  - Current signal: string wrapper/source and primitive target assignment are
    incomplete.

- `Boolean/prototype/toString/S15.6.4.2_A1_T1.js`
  - Expected: `Boolean.prototype.toString()` returns `"false"`.
  - Current signal: primitive prototype objects lack receiver handling compatible
    with ECMA defaults.

- `NativeErrors/RangeError/proto.js`
  - Expected: `Object.getPrototypeOf(RangeError) === Error`.
  - Current signal: native error constructor prototype links are incomplete.

- `Object/defineProperty/* descriptor flag writable mismatch`
  - Expected: descriptor updates preserve and validate flags precisely.
  - Current signal: descriptor validation/apply paths are close but not complete.

- `Function/prototype/bind/15.3.4.5-6-5.js`
  - Expected: bound function call behavior works with script/native callables.
  - Current signal: call/construct is split between syntax evaluator and runtime.

## Architectural Fix Buckets

- Unified runtime `[[Call]]` / `[[Construct]]`.
- Protocol-level `ToPrimitive`, `ToString`, `ToNumber`, and `ToPropertyKey`.
- Primitive wrapper receiver view.
- Declarative intrinsic/built-in property metadata installation.
- Array exotic operations, including `Array.prototype` branding.
- Error constructor/prototype shape table.
- Script/global execution frames with top-level `this` and `arguments`.

