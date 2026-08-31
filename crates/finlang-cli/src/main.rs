use finlang_core::{
    bytecode::{compile as compile_bytecode, Vm},
    compiler::{canonical_debug, compile_source},
    state::FinancialState,
};
use std::{collections::HashMap, env, fs, process};

fn usage() {
    eprintln!("Usage:");
    eprintln!("  finlang <check|compile|format> <file.fin>");
    eprintln!("  finlang run <file.fin> [--feedA <value>] [--feedB <value>]");
    eprintln!("  finlang run <file.fin> [--oracle <name> <value>]");
    eprintln!("  finlang run <file.fin> --live");
    eprintln!("  finlang test");
}

fn source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {}", path, e);
        process::exit(2)
    })
}

fn live_feeds() -> Result<(u64, u64), String> {
    let btc_url = "https://api.coinbase.com/v2/prices/BTC-USD/spot";
    let eth_url = "https://api.coinbase.com/v2/prices/ETH-USD/spot";

    fn fetch_price(url: &str, name: &str) -> Result<u64, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("{} oracle request failed: {}", name, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "{} oracle returned HTTP status {}",
                name,
                response.status()
            ));
        }

        let data: serde_json::Value = response
            .json()
            .map_err(|e| format!("invalid {} oracle response: {}", name, e))?;

        let price = data["data"]["amount"]
            .as_str()
            .ok_or_else(|| format!("missing {} price", name))?
            .parse::<f64>()
            .map_err(|e| format!("invalid {} price: {}", name, e))?;

        if price < 0.0 {
            return Err(format!("{} oracle returned a negative price", name));
        }

        Ok(price.round() as u64)
    }

    let btc = fetch_price(btc_url, "BTC")?;
    let eth = fetch_price(eth_url, "ETH")?;

    Ok((btc, eth))
}

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();

    if command == "--version" || command == "-V" {
        println!("finlang {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if command == "--help" || command == "-h" {
        usage();
        return;
    }

    if command == "test" {
        let status = process::Command::new("cargo")
            .arg("test")
            .status()
            .expect("failed to invoke cargo");

        process::exit(status.code().unwrap_or(1));
    }

    // باقي الكود كما هو
    let path = match args.next() {
        Some(p) => p,
        None => {
            usage();
            process::exit(2);
        }
    };

    let text = source(&path);

    match command.as_str() {
        "check" => match compile_source(&text) {
            Ok(c) => println!("OK: {} ({})", path, c.source_version),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },

        "compile" => match compile_source(&text) {
            Ok(c) => println!("{}", canonical_debug(&c)),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },

        "format" => {
            println!("{}", text.trim());
        }

        "run" => {
            let mut feed_a: u64 = 150;
            let mut feed_b: u64 = 200;
            let mut live = false;
            let mut named_oracles: HashMap<String, Vec<u64>> = HashMap::new();

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--feedA" => {
                        let value = args.next().unwrap_or_else(|| {
                            eprintln!("missing value for --feedA");
                            process::exit(2);
                        });

                        feed_a = value.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("invalid value for --feedA: {}", value);
                            process::exit(2);
                        });
                    }

                    "--feedB" => {
                        let value = args.next().unwrap_or_else(|| {
                            eprintln!("missing value for --feedB");
                            process::exit(2);
                        });

                        feed_b = value.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("invalid value for --feedB: {}", value);
                            process::exit(2);
                        });
                    }

                    "--oracle" => {
                        let name = args.next().unwrap_or_else(|| {
                            eprintln!("missing oracle name");
                            process::exit(2);
                        });

                        let value = args.next().unwrap_or_else(|| {
                            eprintln!("missing value for --oracle {}", name);
                            process::exit(2);
                        });

                        let value = value.parse::<u64>().unwrap_or_else(|_| {
                            eprintln!("invalid value for --oracle {}: {}", name, value);
                            process::exit(2);
                        });

                        named_oracles.entry(name).or_default().push(value);
                    }

                    "--live" => {
                        live = true;
                    }

                    _ => {
                        eprintln!("unknown argument: {}", arg);
                        usage();
                        process::exit(2);
                    }
                }
            }

            if live {
                match live_feeds() {
                    Ok((a, b)) => {
                        feed_a = a;
                        feed_b = b;

                        println!("live oracle: FeedA={} FeedB={}", feed_a, feed_b);
                    }

                    Err(e) => {
                        eprintln!("oracle: {}", e);
                        process::exit(1);
                    }
                }
            }

            match compile_source(&text) {
                Ok(c) => {
                    let mut state = FinancialState::new();

                    state.set_balance("pool", 1_000_000);
                    state.set_balance("alice", 0);
                    state.assets.insert("gold".to_string(), "alice".to_string());
                    state.set_balance("liquidation_pool", 0);
                    state.set_balance("surplus_pool", 0);

                    let mut runtime = Vm::new();

                    runtime.set_oracle_value(finlang_core::ast::SourceId::FeedA, feed_a);

                    runtime.set_oracle_value(finlang_core::ast::SourceId::FeedB, feed_b);

                    for (name, values) in named_oracles {
                        for value in values {
                            runtime.set_oracle_value(
                                finlang_core::ast::SourceId::Named(name.clone()),
                                value,
                            );
                        }
                    }

                    let bytecode = compile_bytecode(&c.typed_ast);

                    match runtime.execute(&bytecode, &mut state) {
                        Ok(v) => {
                            println!("result={:?}", v);
                            println!("state={:?}", state);
                        }

                        Err(e) => {
                            eprintln!("runtime: {}", e);
                            process::exit(1);
                        }
                    }
                }

                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }

        _ => {
            usage();
            process::exit(2);
        }
    }
}
