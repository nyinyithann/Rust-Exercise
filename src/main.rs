mod domain;
mod error;
mod graph;
mod utility;

#[macro_use]
extern crate quick_error;

extern crate chrono;
extern crate colored;

use crate::domain::*;
use crate::graph::*;
use colored::*;
use std::io::prelude::*;

use std::io;

fn main() {
    print_help();
    let mut g = Graph::new();
    loop {
        print_prompt();
        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        if handle.read_line(&mut buffer).is_ok() {
            let trimmed_buffer = buffer.trim();
            if !trimmed_buffer.is_empty() {
                let args: Vec<&str> = trimmed_buffer.split_whitespace().collect();
                match args[0] {
                    PRICE_UPDATE_CMD => {
                        let ret = utility::validate_price_update_input(&args[1..], &g);
                        match ret {
                            Ok(v) => g.update(&v),
                            Err(e) => println!("{}", e.to_string().red()),
                        }
                    }
                    RATE_REQUEST_CMD => {
                        let ret = utility::validate_exchange_rate_input(&args[1..]);
                        match ret {
                            Ok(v) => display_top_rate_with_paths(&g, &v),
                            Err(e) => println!("{}", e.to_string().red()),
                        }
                    }
                    DISPLAY_NODE_CMD => println!("{:?}", g.get_nodes()),
                    DISPLAY_PATH_CMD => println!("{:?}", g.get_paths()),
                    CLEAR_DATA_CMD => g.clear(),
                    HELP_CMD => print_help(),
                    QUIT_CMD => break,
                    _ => println!("Invalid Command"),
                }
            }
        }
    }
}

const PRICE_UPDATE_CMD: &str = ":u";
const RATE_REQUEST_CMD: &str = ":r";
const DISPLAY_NODE_CMD: &str = ":n";
const DISPLAY_PATH_CMD: &str = ":p";
const CLEAR_DATA_CMD: &str = ":c";
const HELP_CMD: &str = ":h";
const QUIT_CMD: &str = ":q";

fn print_help() {
    let opening = "\n\r\n\r◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇ ◇◇";
    println!("{}", opening.green());
    println!("{}", "Please use the following commands to interact with the program.\n\rPlease note that commands are case-sensitive.\n\r".green());
    println!(
        "{:<25}{}",
        "Commands".green().to_string(),
        "Description".green().to_string()
    );
    println!(
        "{:<16}{}",
        PRICE_UPDATE_CMD,
        &format!(
            "Update price, usage: {} {}",
            PRICE_UPDATE_CMD.yellow().to_string(),
            "2017-11-01T09:42:23+00:00 KRAKEN BTC USD 1000.0 0.0009"
                .yellow()
                .to_string()
        )
    );
    println!(
        "{:<16}{}",
        RATE_REQUEST_CMD,
        &format!(
            "Calculate optimal exchange rate, usage: {} {}",
            RATE_REQUEST_CMD.yellow().to_string(),
            "KRAKEN BTC GDAX USD".yellow().to_string()
        )
    );
    println!(
        "{:<16}{}",
        DISPLAY_NODE_CMD,
        &format!(
            "Display all nodes, usage: {}",
            DISPLAY_NODE_CMD.yellow().to_string()
        )
    );
    println!(
        "{:<16}{}",
        DISPLAY_PATH_CMD,
        &format!(
            "Display all paths, usage: {}",
            DISPLAY_PATH_CMD.yellow().to_string()
        )
    );
    println!(
        "{:<16}{}",
        CLEAR_DATA_CMD,
        &format!(
            "Clear the existing data, usage: {}",
            CLEAR_DATA_CMD.yellow().to_string()
        )
    );
    println!(
        "{:<16}{}",
        HELP_CMD,
        &format!(
            "Display this help, usage: {}",
            HELP_CMD.yellow().to_string()
        )
    );
    println!(
        "{:<16}{}",
        QUIT_CMD,
        &format!("Quit, usage: {}", QUIT_CMD.yellow().to_string())
    );
    println!();

    io::stdout().flush().unwrap();
}

fn print_prompt() {
    print!("{}", "◇◇〉".green().to_string());
    io::stdout().flush().unwrap();
}

fn display_top_rate_with_paths(g: &Graph, req: &ExchangeRateRequest) {
    let result = g.get_top_result(&req);
    match result {
        Ok(v) => {
            let mut h = format!(
                "BEST_RATES_BEGIN {} {} {} {} {}",
                req.source_exchange,
                req.source_currency,
                req.destination_exchange,
                req.destination_currency,
                v.rate
            );
            for p in v.paths {
                h.push_str(&format!("\n\r{}, {}", p.exchange, p.currency));
            }
            h.push_str("\nBEST_RATES_END");
            println!("{}", h.blue().to_string());
        }
        Err(e) => println!("{}", e.to_string().red()),
    }
}
