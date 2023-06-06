use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand};
use fst_file::parse_file;
use tracing::{info, trace, Level};
use tracing_subscriber::EnvFilter;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    /// fst file
    file: PathBuf,
    /// log level
    #[arg(long)]
    log_level: Option<Level>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// shows dump of FST file
    List,
    Show {
        /// the index of the block to show
        block_index: usize,
    },
    Dump {
        /// the index of the block to dump
        block_index: usize,
        /// output file
        output: PathBuf,
    },
    DumpAll,
}

fn main() {
    let args = Args::parse();

    if let Some(level) = args.log_level {
        tracing_subscriber::fmt::fmt()
            .with_env_filter(
                EnvFilter::from_str(&format!("fst_file={0},fst_file_cli={0}", level)).unwrap(),
            )
            .init();
    } else {
        tracing_subscriber::fmt::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    trace!("start of cli");
    info!("cli arguments {args:?}");

    let mut file = File::open(args.file).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let blocks = parse_file(&contents).unwrap();

    match args.command {
        Commands::List => {
            println!("{blocks}");
        }
        Commands::Show { block_index } => todo!(),
        Commands::Dump {
            block_index,
            output,
        } => {
            let mut output_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(output)
                .unwrap();
            let block = blocks.get(block_index).unwrap();
            output_file.write_all(&block.extract_data()).unwrap();
        }
        Commands::DumpAll => todo!(),
    }
}
