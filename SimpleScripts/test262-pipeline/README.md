# Test262 Built-Ins Pipeline

Run the current JBS build against local Test262 built-ins files:

```sh
cargo run --bin jbs_test262_builtins -- --limit 200
```

Useful filters:

```sh
cargo run --bin jbs_test262_builtins -- --contains "Object/hasOwn"
cargo run --bin jbs_test262_builtins -- --contains "Array/isArray" --limit 50
cargo run --bin jbs_test262_builtins -- --include-unsupported --limit 100
```

Outputs:

- `SimpleScripts/test262-pipeline/builtins-results.jsonl`
- `SimpleScripts/test262-pipeline/builtins-summary.md`

The pipeline is intentionally heuristic. It strips Test262 frontmatter, skips
obviously unsupported syntax by default, evaluates each selected file in a fresh
`Runtime`, and records pass/fail/unsupported. It does not yet implement full
frontmatter semantics, negative tests, async tests, modules, or full harness
include loading.
