mod verifier;

use std::env;
use std::fs;

use verifier::types::ProofEnvelope;
use verifier::verify::verify;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 || args[1] != "verify" || args[2] != "--envelope" {
        eprintln!("usage: tscp-cli verify --envelope <file>");
        std::process::exit(2);
    }

    let path = &args[3];

    let data = match fs::read_to_string(path) {
        Ok(v) => v,
        Err(_) => {
            println!(r#"{{"status":"FAIL","reasoncode":"REJECTSCHEMA"}}"#);
            std::process::exit(1);
        }
    };

    let envelope: ProofEnvelope = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => {
            println!(r#"{{"status":"FAIL","reasoncode":"REJECTSCHEMA"}}"#);
            std::process::exit(1);
        }
    };

    let result = verify(&envelope);

    println!("{}", serde_json::to_string(&result).unwrap());

    std::process::exit(if result.status == "PASS" { 0 } else { 1 });
}
