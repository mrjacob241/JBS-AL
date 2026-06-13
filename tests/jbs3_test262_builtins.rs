use std::fs;
use std::path::Path;

use jbs::Runtime;

#[derive(Debug, PartialEq)]
enum Status {
    Pass,
    ExpectedFail,
    Unsupported,
    ExpectedUnavailable,
}

#[test]
fn curated_test262_builtins_are_tracked() {
    let manifest = Path::new("SimpleScripts/JBS3Builtins/test262-builtins.manifest");
    let content = fs::read_to_string(manifest).unwrap();
    let mut count = 0;

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<_> = line.split('|').collect();
        assert_eq!(parts.len(), 3, "bad manifest line: {line}");
        let path = Path::new(parts[0]);
        let status = parse_status(parts[1]);

        if !path.exists() {
            assert_eq!(
                status,
                Status::ExpectedUnavailable,
                "{} is missing but was not marked expected-unavailable",
                path.display()
            );
            count += 1;
            continue;
        }

        match status {
            Status::Pass => {
                let source = fs::read_to_string(path).unwrap();
                let stripped = strip_test262_frontmatter(&source);
                let mut runtime = Runtime::new();
                runtime
                    .eval_script(&stripped)
                    .unwrap_or_else(|error| panic!("{} failed: {}", path.display(), error));
            }
            Status::ExpectedFail => {
                let source = fs::read_to_string(path).unwrap();
                let stripped = strip_test262_frontmatter(&source);
                let mut runtime = Runtime::new();
                assert!(
                    runtime.eval_script(&stripped).is_err(),
                    "{} unexpectedly passed; promote or replace this Test262 row",
                    path.display()
                );
            }
            Status::Unsupported => {}
            Status::ExpectedUnavailable => {
                panic!(
                    "{} exists but is marked expected-unavailable",
                    path.display()
                );
            }
        }
        count += 1;
    }

    assert_eq!(count, 33);
}

fn parse_status(raw: &str) -> Status {
    match raw {
        "pass" => Status::Pass,
        "expected-fail" => Status::ExpectedFail,
        "unsupported" => Status::Unsupported,
        "expected-unavailable" => Status::ExpectedUnavailable,
        other => panic!("unknown Test262 status: {other}"),
    }
}

fn strip_test262_frontmatter(source: &str) -> String {
    let mut out = String::new();
    let mut index = 0;
    while let Some(start) = source[index..].find("/*") {
        let absolute_start = index + start;
        out.push_str(&source[index..absolute_start]);
        let Some(end) = source[absolute_start + 2..].find("*/") else {
            out.push_str(&source[absolute_start..]);
            return out;
        };
        index = absolute_start + 2 + end + 2;
    }
    out.push_str(&source[index..]);
    out
}
