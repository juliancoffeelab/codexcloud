use hex_command_core::{model::Rules, simulation};
use std::io::{self, BufRead, Write};
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--fingerprint") {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../data/rules.json")).unwrap();
        println!("{}", data["sourceFingerprint"].as_str().unwrap());
        return;
    }
    let rules = Rules::load();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for line in stdin.lock().lines() {
        let line = line.expect("input");
        if line.trim().is_empty() {
            continue;
        }
        let request = serde_json::from_str(&line).expect("JSON config");
        let result = simulation::run(rules.clone(), request);
        serde_json::to_writer(&mut output, &result).expect("output");
        writeln!(output).unwrap();
        output.flush().unwrap();
    }
}
