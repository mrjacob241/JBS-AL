use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use jbs::Runtime;

const TEST_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Default)]
struct Config {
    root: PathBuf,
    out_dir: PathBuf,
    contains: Option<String>,
    limit: Option<usize>,
    include_unsupported: bool,
    stop_on_fail: bool,
    eval_one: Option<PathBuf>,
}

#[derive(Default)]
struct Counts {
    total_seen: usize,
    selected: usize,
    passed: usize,
    failed: usize,
    timed_out: usize,
    unsupported: usize,
}

fn main() {
    let config = parse_args();
    let result = if let Some(path) = config.eval_one {
        eval_one_file(&path)
    } else {
        run(config)
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn eval_one_file(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let stripped = prepare_test262_source(path, &source)?;
    let mut runtime = Runtime::new();
    match runtime.eval_script(&stripped) {
        Ok(value) => {
            println!("{value:?}");
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn run(config: Config) -> Result<(), String> {
    let started = Instant::now();
    let files = collect_js_files(&config.root)?;
    fs::create_dir_all(&config.out_dir).map_err(|error| {
        format!(
            "failed to create report directory {}: {error}",
            config.out_dir.display()
        )
    })?;

    let jsonl_path = config.out_dir.join("builtins-results.jsonl");
    let md_path = config.out_dir.join("builtins-summary.md");
    let mut jsonl = BufWriter::new(
        File::create(&jsonl_path)
            .map_err(|error| format!("failed to create {}: {error}", jsonl_path.display()))?,
    );
    let mut counts = Counts::default();
    let mut first_failures = Vec::new();

    for path in files {
        counts.total_seen += 1;
        let display = path.to_string_lossy().replace('\\', "/");
        if let Some(needle) = &config.contains {
            if !display.contains(needle) {
                continue;
            }
        }
        if let Some(limit) = config.limit {
            if counts.selected >= limit {
                break;
            }
        }
        counts.selected += 1;

        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let metadata = extract_frontmatter(&source);
        let stripped = strip_test262_frontmatter(&source);
        let unsupported_scan_source = strip_line_comments(&stripped);
        let unsupported = unsupported_reason(&display, &metadata, &unsupported_scan_source);

        if unsupported.is_some() && !config.include_unsupported {
            counts.unsupported += 1;
            write_jsonl(
                &mut jsonl,
                &display,
                "unsupported",
                unsupported
                    .as_deref()
                    .unwrap_or("unsupported metadata or syntax"),
            )?;
            continue;
        }

        match eval_with_timeout(&path, TEST_TIMEOUT)? {
            TestOutcome::Pass(detail) => {
                counts.passed += 1;
                write_jsonl(&mut jsonl, &display, "pass", &detail)?;
            }
            TestOutcome::Fail(message) => {
                counts.failed += 1;
                if first_failures.len() < 25 {
                    first_failures.push((display.clone(), message.clone()));
                }
                write_jsonl(&mut jsonl, &display, "fail", &message)?;
                if config.stop_on_fail {
                    break;
                }
            }
            TestOutcome::Timeout(message) => {
                counts.timed_out += 1;
                if first_failures.len() < 25 {
                    first_failures.push((display.clone(), message.clone()));
                }
                write_jsonl(&mut jsonl, &display, "timeout", &message)?;
                if config.stop_on_fail {
                    break;
                }
            }
        }
    }
    jsonl
        .flush()
        .map_err(|error| format!("failed to flush {}: {error}", jsonl_path.display()))?;

    let mut summary = String::new();
    summary.push_str("# JBS Test262 Built-Ins Pipeline Report\n\n");
    summary.push_str(&format!("- Root: `{}`\n", config.root.display()));
    summary.push_str(&format!(
        "- Filter: `{}`\n",
        config.contains.as_deref().unwrap_or("*")
    ));
    summary.push_str(&format!(
        "- Limit: `{}`\n",
        config
            .limit
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_owned())
    ));
    summary.push_str(&format!(
        "- Include unsupported: `{}`\n",
        config.include_unsupported
    ));
    summary.push_str(&format!(
        "- Per-test timeout ms: `{}`\n",
        TEST_TIMEOUT.as_millis()
    ));
    summary.push_str(&format!(
        "- Duration ms: `{}`\n\n",
        started.elapsed().as_millis()
    ));
    summary.push_str("## Counts\n\n");
    summary.push_str(&format!("- Seen: `{}`\n", counts.total_seen));
    summary.push_str(&format!("- Selected: `{}`\n", counts.selected));
    summary.push_str(&format!("- Passed: `{}`\n", counts.passed));
    summary.push_str(&format!("- Failed: `{}`\n", counts.failed));
    summary.push_str(&format!("- Timed out: `{}`\n", counts.timed_out));
    summary.push_str(&format!(
        "- Unsupported skipped: `{}`\n\n",
        counts.unsupported
    ));
    summary.push_str("## First Failures\n\n");
    if first_failures.is_empty() {
        summary.push_str("No failures recorded.\n");
    } else {
        for (path, error) in first_failures {
            summary.push_str(&format!("- `{path}`: `{}`\n", escape_md(&error)));
        }
    }
    fs::write(&md_path, summary)
        .map_err(|error| format!("failed to write {}: {error}", md_path.display()))?;

    println!("wrote {}", jsonl_path.display());
    println!("wrote {}", md_path.display());
    println!(
        "selected={} pass={} fail={} timeout={} unsupported={}",
        counts.selected, counts.passed, counts.failed, counts.timed_out, counts.unsupported
    );
    Ok(())
}

fn parse_args() -> Config {
    let mut config = Config {
        root: PathBuf::from("ECMAScript/test262-main/test/built-ins"),
        out_dir: PathBuf::from("SimpleScripts/test262-pipeline"),
        ..Config::default()
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => config.root = PathBuf::from(next_arg(&mut args, "--root")),
            "--out" => config.out_dir = PathBuf::from(next_arg(&mut args, "--out")),
            "--contains" => config.contains = Some(next_arg(&mut args, "--contains")),
            "--limit" => {
                config.limit = Some(
                    next_arg(&mut args, "--limit")
                        .parse()
                        .expect("--limit must be a positive integer"),
                );
            }
            "--include-unsupported" => config.include_unsupported = true,
            "--stop-on-fail" => config.stop_on_fail = true,
            "--eval-one" => {
                config.eval_one = Some(PathBuf::from(next_arg(&mut args, "--eval-one")))
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => panic!("unknown argument: {other}"),
        }
    }
    config
}

fn next_arg(args: &mut impl Iterator<Item = String>, name: &str) -> String {
    args.next()
        .unwrap_or_else(|| panic!("{name} requires a value"))
}

fn print_help() {
    println!(
        "Usage: cargo run --bin jbs_test262_builtins -- [--root PATH] [--out PATH] [--contains TEXT] [--limit N] [--include-unsupported] [--stop-on-fail]"
    );
}

enum TestOutcome {
    Pass(String),
    Fail(String),
    Timeout(String),
}

fn eval_with_timeout(path: &Path, timeout: Duration) -> Result<TestOutcome, String> {
    let executable =
        env::current_exe().map_err(|error| format!("failed to resolve current exe: {error}"))?;
    let mut child = Command::new(executable)
        .arg("--eval-one")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn test child for {}: {error}", path.display()))?;

    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to poll test child: {error}"))?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| format!("failed to collect test child output: {error}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            if output.status.success() {
                return Ok(TestOutcome::Pass(stdout));
            }
            let detail = if stderr.is_empty() { stdout } else { stderr };
            return Ok(TestOutcome::Fail(detail));
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .map_err(|error| format!("failed to collect timed-out child output: {error}"))?;
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = if stderr.is_empty() {
                format!("timeout after {} ms", timeout.as_millis())
            } else {
                format!("timeout after {} ms: {stderr}", timeout.as_millis())
            };
            return Ok(TestOutcome::Timeout(detail));
        }

        thread::sleep(Duration::from_millis(5));
    }
}

fn collect_js_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_js_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_js_files_inner(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("failed to stat {}: {error}", path.display()))?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|error| format!("failed to read directory {}: {error}", path.display()))?
        {
            let entry =
                entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
            collect_js_files_inner(&entry.path(), files)?;
        }
    } else if path.extension().and_then(|value| value.to_str()) == Some("js") {
        files.push(path.to_owned());
    }
    Ok(())
}

fn extract_frontmatter(source: &str) -> String {
    let Some(start) = source.find("/*---") else {
        return String::new();
    };
    let Some(end) = source[start + 5..].find("---*/") else {
        return String::new();
    };
    source[start + 5..start + 5 + end].to_owned()
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

fn prepare_test262_source(path: &Path, source: &str) -> Result<String, String> {
    let metadata = extract_frontmatter(source);
    let mut prepared = String::new();
    if metadata_has_flag(&metadata, "onlyStrict") {
        prepared.push_str("\"use strict\";\n");
    }
    prepared.push_str(test262_host_prelude());
    for include in extract_includes(&metadata) {
        let include_path = test262_harness_dir(path).join(&include);
        let include_source = fs::read_to_string(&include_path).map_err(|error| {
            format!(
                "failed to read Test262 harness include {} for {}: {error}",
                include_path.display(),
                path.display()
            )
        })?;
        if include == "promiseHelper.js" {
            prepared.push_str(test262_promise_helper_legacy_subset());
        } else {
            prepared.push_str(&strip_test262_frontmatter(&include_source));
        }
        prepared.push('\n');
    }
    prepared.push_str(&strip_test262_frontmatter(source));
    Ok(prepared)
}

fn test262_host_prelude() -> &'static str {
    "var $262 = {
  createRealm: function () {
    return { global: globalThis };
  }
};
"
}

fn test262_promise_helper_legacy_subset() -> &'static str {
    "function checkSequence(arr, message) {
  arr.forEach(function(e, i) {
    if (e !== (i + 1)) {
      throw new Test262Error((message ? message : \"Steps in unexpected sequence:\") +
             \" '\" + arr.join(',') + \"'\");
    }
  });
  return true;
}
"
}

fn extract_includes(metadata: &str) -> Vec<String> {
    let Some(includes_start) = metadata.find("includes:") else {
        return Vec::new();
    };
    let after_key = &metadata[includes_start + "includes:".len()..];
    let Some(open) = after_key.find('[') else {
        return Vec::new();
    };
    let after_open = &after_key[open + 1..];
    let Some(close) = after_open.find(']') else {
        return Vec::new();
    };
    after_open[..close]
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.trim_matches('"').trim_matches('\'').to_owned())
        .collect()
}

fn metadata_has_flag(metadata: &str, flag: &str) -> bool {
    let Some(flags_start) = metadata.find("flags:") else {
        return false;
    };
    let after_key = &metadata[flags_start + "flags:".len()..];
    let Some(open) = after_key.find('[') else {
        return false;
    };
    let after_open = &after_key[open + 1..];
    let Some(close) = after_open.find(']') else {
        return false;
    };
    after_open[..close]
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == flag)
}

fn test262_harness_dir(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("test262-main") {
            return ancestor.join("harness");
        }
    }
    PathBuf::from("ECMAScript/test262-main/harness")
}

fn strip_line_comments(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        if let Some((prefix, _)) = line.split_once("//") {
            out.push_str(prefix);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn unsupported_reason(path: &str, metadata: &str, source: &str) -> Option<String> {
    let unsupported_path_needles = [
        ("/ArrayBuffer/", "ArrayBuffer built-ins are outside JBS-5"),
        ("/Atomics/", "Atomics built-ins are future JBS surface"),
        (
            "/AsyncDisposableStack/",
            "explicit resource management is future JBS surface",
        ),
        (
            "/AsyncFunction/",
            "async function built-ins are future JBS surface",
        ),
        (
            "/AsyncGeneratorFunction/",
            "async generator built-ins are future JBS surface",
        ),
        ("/DataView/", "DataView built-ins are outside JBS-5"),
        (
            "/DisposableStack/",
            "explicit resource management is future JBS surface",
        ),
        (
            "/FinalizationRegistry/",
            "FinalizationRegistry built-ins are future JBS surface",
        ),
        ("/Map/", "Map runtime built-ins are outside JBS-5"),
        ("/Promise/", "Promise/job built-ins are outside JBS-5"),
        ("/Set/", "Set runtime built-ins are outside JBS-5"),
        (
            "/SharedArrayBuffer/",
            "SharedArrayBuffer built-ins are outside JBS-5",
        ),
        ("/Temporal/", "Temporal built-ins are future JBS surface"),
        ("/Symbol/", "Symbol built-ins are outside JBS-5"),
        ("Symbol.species", "Symbol.species tests are outside JBS-5"),
        ("/TypedArray/", "TypedArray built-ins are outside JBS-5"),
        (
            "/TypedArrayConstructors/",
            "typed arrays are future JBS surface",
        ),
        ("/Int8Array/", "typed arrays are future JBS surface"),
        ("/Int16Array/", "typed arrays are future JBS surface"),
        ("/Int32Array/", "typed arrays are future JBS surface"),
        ("/Uint8Array/", "typed arrays are future JBS surface"),
        ("/Uint8ClampedArray/", "typed arrays are future JBS surface"),
        ("/Uint16Array/", "typed arrays are future JBS surface"),
        ("/Uint32Array/", "typed arrays are future JBS surface"),
        ("/Float32Array/", "typed arrays are future JBS surface"),
        ("/Float64Array/", "typed arrays are future JBS surface"),
        ("/BigInt64Array/", "typed arrays are future JBS surface"),
        ("/BigUint64Array/", "typed arrays are future JBS surface"),
        ("/WeakRef/", "WeakRef built-ins are future JBS surface"),
        ("/WeakMap/", "WeakMap built-ins are outside JBS-5"),
        ("/WeakSet/", "WeakSet built-ins are outside JBS-5"),
        (
            "/Array/prototype/concat/",
            "Array.prototype.concat is outside JBS-5",
        ),
        (
            "/Array/prototype/map/",
            "Array.prototype.map Test262 coverage is outside JBS-7",
        ),
        (
            "/Array/prototype/filter/",
            "Array.prototype.filter Test262 coverage is outside JBS-7",
        ),
        (
            "/Array/prototype/forEach/",
            "Array.prototype.forEach Test262 coverage is outside JBS-7",
        ),
        (
            "/Array/prototype/reduce/",
            "Array.prototype.reduce Test262 coverage is outside JBS-7",
        ),
    ];
    for (needle, reason) in unsupported_path_needles {
        if path.contains(needle) {
            return Some(reason.to_owned());
        }
    }

    let meta_needles = [
        ("negative:", "negative test metadata"),
        ("flags: [module", "module test"),
        ("module", "module test"),
        ("async", "async test"),
        ("features: [async", "async feature"),
        ("features: [generators", "generator feature"),
    ];
    for (needle, reason) in meta_needles {
        if metadata.contains(needle) {
            return Some(reason.to_owned());
        }
    }

    let future_global_needles = [
        (
            "ArrayBuffer",
            "ArrayBuffer and typed arrays are future JBS surface",
        ),
        (
            "SharedArrayBuffer",
            "SharedArrayBuffer is future JBS surface",
        ),
        ("DataView", "DataView is future JBS surface"),
        ("Float32Array", "typed arrays are future JBS surface"),
        ("Float64Array", "typed arrays are future JBS surface"),
        ("Int8Array", "typed arrays are future JBS surface"),
        ("Int16Array", "typed arrays are future JBS surface"),
        ("Int32Array", "typed arrays are future JBS surface"),
        ("Uint8Array", "typed arrays are future JBS surface"),
        ("Uint8ClampedArray", "typed arrays are future JBS surface"),
        ("Uint16Array", "typed arrays are future JBS surface"),
        ("Uint32Array", "typed arrays are future JBS surface"),
        ("BigInt64Array", "typed arrays are future JBS surface"),
        ("BigUint64Array", "typed arrays are future JBS surface"),
        ("Atomics", "Atomics is future JBS surface"),
        ("WeakRef", "WeakRef is future JBS surface"),
        (
            "FinalizationRegistry",
            "FinalizationRegistry is future JBS surface",
        ),
        (
            "AsyncDisposableStack",
            "explicit resource management is future JBS surface",
        ),
        (
            "DisposableStack",
            "explicit resource management is future JBS surface",
        ),
    ];
    for (needle, reason) in future_global_needles {
        if source.contains(needle) {
            return Some(reason.to_owned());
        }
    }

    let syntax_needles = [
        ("=>", "arrow function"),
        (" class ", "class syntax"),
        ("class ", "class syntax"),
        (" import ", "module import"),
        (" export ", "module export"),
        ("`", "template literal"),
        ("...", "spread/rest"),
        ("?.", "optional chaining"),
        ("??", "nullish coalescing"),
        ("function*", "generator function"),
        ("async function", "async function"),
        ("for (", "possibly unsupported loop form"),
        ("for(", "possibly unsupported loop form"),
    ];
    for (needle, reason) in syntax_needles {
        if source.contains(needle) {
            return Some(reason.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_test262_includes_from_frontmatter() {
        let metadata = "
description: sample
includes: [propertyHelper.js, proxyTrapsHelper.js]
features: [Proxy]
";
        assert_eq!(
            extract_includes(metadata),
            vec![
                "propertyHelper.js".to_owned(),
                "proxyTrapsHelper.js".to_owned()
            ]
        );
    }

    #[test]
    fn prepares_source_with_host_prelude_and_harness_includes() {
        let path =
            Path::new("ECMAScript/test262-main/test/built-ins/Proxy/has/call-in-prototype.js");
        let source = fs::read_to_string(path).unwrap();
        let prepared = prepare_test262_source(path, &source).unwrap();

        assert!(prepared.contains("var $262 ="));
        assert!(prepared.contains("function allowProxyTraps(overrides)"));
        assert!(!prepared.contains("includes: [proxyTrapsHelper.js]"));
    }

    #[test]
    fn promise_helper_include_keeps_legacy_check_sequence_parseable() {
        let path = Path::new(
            "ECMAScript/test262-main/test/built-ins/AggregateError/order-of-args-evaluation.js",
        );
        let source = fs::read_to_string(path).unwrap();
        let prepared = prepare_test262_source(path, &source).unwrap();

        assert!(prepared.contains("function checkSequence(arr, message)"));
        assert!(!prepared.contains("checkSettledPromises"));
        assert!(!prepared.contains("=>"));
    }

    #[test]
    fn future_typed_array_surface_is_classified_as_unsupported() {
        assert_eq!(
            unsupported_reason(
                "ECMAScript/test262-main/test/built-ins/Array/from/items-is-arraybuffer.js",
                "",
                "var buffer = new ArrayBuffer(8);"
            )
            .as_deref(),
            Some("ArrayBuffer and typed arrays are future JBS surface")
        );
        assert_eq!(
            unsupported_reason(
                "ECMAScript/test262-main/test/built-ins/Array/prototype/flatMap/array-like-objects-typedarrays.js",
                "",
                "var sample = new Int32Array([1, 2]);"
            )
            .as_deref(),
            Some("typed arrays are future JBS surface")
        );
        assert_eq!(
            unsupported_reason(
                "ECMAScript/test262-main/test/built-ins/TypedArrayConstructors/prototype/values/inherited.js",
                "",
                ""
            )
            .as_deref(),
            Some("typed arrays are future JBS surface")
        );
        assert_eq!(
            unsupported_reason(
                "ECMAScript/test262-main/test/built-ins/Temporal/Now/instant.js",
                "",
                ""
            )
            .as_deref(),
            Some("Temporal built-ins are future JBS surface")
        );
    }
}

fn write_jsonl(
    writer: &mut BufWriter<File>,
    path: &str,
    status: &str,
    detail: &str,
) -> Result<(), String> {
    writeln!(
        writer,
        "{{\"path\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
        escape_json(path),
        status,
        escape_json(detail)
    )
    .map_err(|error| format!("failed to write jsonl: {error}"))
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn escape_md(value: &str) -> String {
    value.replace('`', "'").replace('\n', " ")
}
