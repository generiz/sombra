use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use sombra::{AttemptOutcome, Bundle, BundleStore, EnqueueOutcome, Priority, Scenario, Simulator};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "sombra", version, about = "Resilient multi-transport messaging research prototype")]
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
    Queue {
        #[command(subcommand)]
        command: QueueCommand,
    },
}

#[derive(Subcommand)]
enum QueueCommand {
    Enqueue {
        #[arg(long, default_value = "sombra-store.json")]
        file: PathBuf,
        #[arg(long)]
        envelope: PathBuf,
        #[arg(long, value_enum, default_value = "routine")]
        priority: PriorityArg,
        #[arg(long, default_value_t = 86_400)]
        ttl_secs: u64,
        #[arg(long, default_value_t = 8)]
        hop_limit: u8,
    },
    Next {
        #[arg(long, default_value = "sombra-store.json")]
        file: PathBuf,
        #[arg(long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    Attempt {
        #[arg(long, default_value = "sombra-store.json")]
        file: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        delivered: bool,
    },
    Prune {
        #[arg(long, default_value = "sombra-store.json")]
        file: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PriorityArg {
    Routine,
    Important,
    Urgent,
}

impl From<PriorityArg> for Priority {
    fn from(value: PriorityArg) -> Self {
        match value {
            PriorityArg::Routine => Priority::Routine,
            PriorityArg::Important => Priority::Important,
            PriorityArg::Urgent => Priority::Urgent,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Simulate { scenario, json } => run_simulation(scenario, json)?,
        Command::Queue { command } => run_queue(command)?,
    }
    Ok(())
}

fn run_simulation(scenario: Option<PathBuf>, json: bool) -> Result<()> {
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
        println!(
            "delivered: {} ({:.1}%)",
            report.delivered,
            report.delivery_ratio * 100.0
        );
        println!("mean latency: {} ms", report.mean_latency_ms);
        for (transport, count) in report.by_transport {
            println!("{transport}: {count}");
        }
    }
    Ok(())
}

fn run_queue(command: QueueCommand) -> Result<()> {
    match command {
        QueueCommand::Enqueue {
            file,
            envelope,
            priority,
            ttl_secs,
            hop_limit,
        } => {
            let opaque_bytes = fs::read(&envelope)?;
            let bundle = Bundle::new(
                &opaque_bytes,
                Duration::from_secs(ttl_secs),
                hop_limit,
                priority.into(),
            );
            let id = bundle.id.clone();
            let mut store = BundleStore::load(&file)?;
            let outcome = store.enqueue(bundle, opaque_bytes, now_ms());
            store.save()?;
            println!("id: {id}");
            println!("outcome: {}", enqueue_label(&outcome));
            println!("queued: {}", store.len());
        }
        QueueCommand::Next { file, limit, json } => {
            let store = BundleStore::load(&file)?;
            let ready = store.ready(now_ms(), limit);
            if json {
                println!("{}", serde_json::to_string_pretty(&ready)?);
            } else if ready.is_empty() {
                println!("no bundle ready");
            } else {
                for item in ready {
                    println!(
                        "{} {:?} bytes={} attempts={} hops={} next={}",
                        item.bundle.id,
                        item.bundle.priority,
                        item.envelope.len(),
                        item.attempts,
                        item.bundle.hop_limit,
                        item.next_attempt_at_ms
                    );
                }
            }
        }
        QueueCommand::Attempt {
            file,
            id,
            delivered,
        } => {
            let mut store = BundleStore::load(&file)?;
            let outcome = store.record_attempt(&id, now_ms(), delivered);
            store.save()?;
            match outcome {
                AttemptOutcome::Delivered => println!("delivered: {id}"),
                AttemptOutcome::Deferred { next_attempt_at_ms } => {
                    println!("deferred: {id}");
                    println!("next_attempt_at_ms: {next_attempt_at_ms}");
                }
                AttemptOutcome::Missing => println!("missing: {id}"),
            }
        }
        QueueCommand::Prune { file } => {
            let mut store = BundleStore::load(&file)?;
            let removed = store.prune_expired(now_ms());
            store.save()?;
            println!("removed: {removed}");
            println!("queued: {}", store.len());
        }
    }
    Ok(())
}

fn enqueue_label(outcome: &EnqueueOutcome) -> &'static str {
    match outcome {
        EnqueueOutcome::Stored => "stored",
        EnqueueOutcome::Duplicate => "duplicate",
        EnqueueOutcome::Expired => "expired",
        EnqueueOutcome::Full => "full",
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
