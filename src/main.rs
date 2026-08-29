use anyhow::Result;
use clap::{Parser, Subcommand};
use sombra::{Scenario, Simulator};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sombra", version, about = "Resilient multi-transport network simulator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Simulate {
        #[arg(long)]
        scenario: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Simulate { scenario, json } => {
            let scenario = match scenario {
                Some(path) => serde_json::from_str::<Scenario>(&fs::read_to_string(path)?)?,
                None => Scenario::default(),
            };
            let report = Simulator::default().run(&scenario);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("nodes: {}", report.nodes);
                println!("messages: {}", report.messages);
                println!("delivered: {} ({:.1}%)", report.delivered, report.delivery_ratio * 100.0);
                println!("mean latency: {} ms", report.mean_latency_ms);
                for (transport, count) in report.by_transport {
                    println!("{transport}: {count}");
                }
            }
        }
    }
    Ok(())
}
