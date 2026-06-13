# JBS-7 Built-ins Regressions

Focused scripts for built-ins failures found while comparing current JBS-7 behavior against nearby Test262 cases.

These are intentionally narrow:
- `Array.prototype` brand and length.
- `Object.assign` primitive target boxing and string source enumeration.
- Primitive prototype default wrapper values.
- Native error constructor inheritance.
