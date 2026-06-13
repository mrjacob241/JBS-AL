# JBS Test Layout

- `jbs0_object_kernel.rs`: normal passing tests for implemented JBS-0 behavior.
- `expected_failures.rs`: ignored executable backlog harness for known ECMA
  gaps; currently empty after Block 0-9 stabilization.

Default check:

```sh
cargo test
```

Failure-backlog check:

```sh
cargo test --test expected_failures -- --ignored
```

When an ignored test starts passing because the implementation caught up, remove
`#[ignore]` and move the case into the normal suite. This keeps known failures
visible without making the default build red.
