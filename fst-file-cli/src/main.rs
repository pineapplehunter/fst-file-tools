use std::{
    fmt::{self},
    fs::{File, OpenOptions},
    io::{IsTerminal, Read, Write},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand, ValueEnum};

use fst_file::parse_file;
use once_cell::sync::Lazy;
use termion::color;
use tracing::{debug, trace, Level};
use tracing_subscriber::EnvFilter;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
    /// log level
    #[arg(long)]
    log_level: Option<Level>,
}

#[derive(Debug, Args)]
struct CommonArgs {
    /// input fst file
    input_file: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows all blocks in a FST file.
    List {
        #[command(flatten)]
        common: CommonArgs,
        /// output file format
        #[arg(long, value_enum, default_value_t)]
        output_format: OutputFormat,
    },
    Show {
        #[command(flatten)]
        common: CommonArgs,
        /// the index of the block to show
        block_index: usize,
    },
    /// Dump the contents of a block to a file.
    /// If the block data is compressed, it will first uncompress the data and dump the contents.
    Dump {
        #[command(flatten)]
        common: CommonArgs,
        /// the index of the block to dump
        #[arg(short = 'i', long = "index")]
        block_index: usize,
        /// output file to dump the contents
        output_file: PathBuf,
    },
    DumpAll {
        #[command(flatten)]
        common: CommonArgs,
    },
}

impl CliArgs {
    fn get_common(&self) -> &CommonArgs {
        match &self.command {
            Commands::List { common, .. } => common,
            Commands::Show { common, .. } => common,
            Commands::Dump { common, .. } => common,
            Commands::DumpAll { common } => common,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// for humans reading on the terminal
    #[default]
    PlainText,
    /// for other programs to read
    Json,
    /// for debugging?
    PrettyJson,
}

static IS_TERMINAL: Lazy<bool> = Lazy::new(|| std::io::stdout().is_terminal());
trait OnlyOnTerminal: Sized + fmt::Display {
    fn only_on_terminal(&self) -> String {
        let mut s = Vec::new();
        if *IS_TERMINAL {
            write!(s, "{}", self).unwrap();
        }
        String::from_utf8_lossy(&s).to_string()
    }
}

impl<T> OnlyOnTerminal for T where T: fmt::Display {}

fn main() {
    let args = CliArgs::parse();
    if let Some(level) = args.log_level {
        tracing_subscriber::fmt::fmt()
            .with_writer(|| std::io::stderr())
            .with_max_level(level)
            .init();
    } else {
        tracing_subscriber::fmt::fmt()
            .with_writer(|| std::io::stderr())
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    trace!("start of cli");
    debug!("cli arguments {args:?}");

    let common = args.get_common();

    let mut file = File::open(&common.input_file).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let blocks = parse_file(&contents).unwrap();

    match args.command {
        Commands::List { output_format, .. } => match output_format {
            OutputFormat::PlainText => {
                for (idx, block) in blocks.iter().enumerate() {
                    println!(
                        "Block#{idx} {bold}{}{reset}",
                        block.get_block().block_type,
                        bold = termion::style::Bold.only_on_terminal(),
                        reset = termion::style::Reset.only_on_terminal()
                    );
                    println!(
                        "    block offset: {blue}{}{reset}",
                        block.get_block_start_offset(),
                        blue = color::Fg(color::Blue).only_on_terminal(),
                        reset = color::Fg(color::Reset).only_on_terminal()
                    );
                    println!(
                        "    block length: {yello}{}{reset}",
                        block.get_block_length(),
                        yello = color::Fg(color::Yellow).only_on_terminal(),
                        reset = color::Fg(color::Reset).only_on_terminal()
                    );
                    println!(
                        "    data offset:  {green}{}{reset}",
                        block.get_block_start_offset() + 9,
                        green = color::Fg(color::Green).only_on_terminal(),
                        reset = color::Fg(color::Reset).only_on_terminal()
                    );
                    println!(
                        "    data length:  {red}{}{reset}",
                        block.get_data_length(),
                        red = color::Fg(color::Red).only_on_terminal(),
                        reset = color::Fg(color::Reset).only_on_terminal()
                    );
                    println!(
                        "    block end:    {cyan}{}{reset}",
                        block.get_block_end_offset(),
                        cyan = color::Fg(color::Cyan).only_on_terminal(),
                        reset = color::Fg(color::Reset).only_on_terminal()
                    );
                }
            }
            OutputFormat::Json => {
                let json = serde_json::to_string(&blocks).unwrap();
                print!("{}", json);
            }
            OutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&blocks).unwrap();
                print!("{}", json);
            }
        },
        Commands::Show { .. } => todo!(),
        Commands::Dump {
            block_index,
            output_file: output,
            ..
        } => {
            let mut output_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(output)
                .unwrap();
            let block_info = blocks.get(block_index).unwrap();
            output_file
                .write_all(&block_info.get_block().extract_data())
                .unwrap();
        }
        Commands::DumpAll { .. } => todo!(),
    }
}
