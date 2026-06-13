use jbs::Runtime;
use std::{env, fs, process};

fn main() {
    let mut runtime = Runtime::new();
    if let Some(path) = env::args().nth(1) {
        let source = fs::read_to_string(&path).unwrap_or_else(|error| {
            eprintln!("failed to read {path}: {error}");
            process::exit(1);
        });
        match runtime.eval_script(&source) {
            Ok(value) => println!("{value:?}"),
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    } else {
        let realm = runtime.default_realm();
        println!(
            "JustBarelyScript JBS-0 runtime initialized: realm {}",
            realm.index()
        );
    }
}
