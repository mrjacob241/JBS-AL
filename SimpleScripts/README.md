# JBS-0 Basilar Script Collection

This folder contains 50 plain ECMAScript JavaScript scripts that the current
JBS-0 parser/evaluator can execute.

The scripts intentionally stay within the JBS-0 subset:

- expression statements
- `var`/`let`/`const` declarations with initializers
- identifiers
- number, string, boolean, `null`, and `undefined` literals
- object literals with simple property definitions
- dot property access
- function/method calls
- unary `!`
- the currently implemented `Object` constructor/static methods

Expected final values are stored in `manifest.txt`; the scripts themselves do
not use custom syntax or custom assertion helpers.

Run the whole collection:

```sh
cargo test --test jbs0_script_collection
```

Run one script through the JBS binary:

```sh
cargo run -- SimpleScripts/015_object_call_define_x.js
```
