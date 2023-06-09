use std::{
    borrow::Cow,
    collections::HashMap,
    fmt,
    fs::{File, OpenOptions},
    io::{IsTerminal, Read, Write},
    path::PathBuf,
    sync::OnceLock,
};

use clap::{Args, Parser, Subcommand, ValueEnum};

use termion::color;
use tracing::{debug, debug_span, error, metadata::LevelFilter, trace};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::__tracing_subscriber_SubscriberExt};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Commands,
    /// log level
    #[arg(global = true, short, long, value_enum, default_value_t)]
    log_level: ArgLevel,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ArgLevel {
    Trace,
    Debug,
    Info,
    #[default]
    Warn,
    Error,
    Off,
}

impl From<ArgLevel> for LevelFilter {
    fn from(value: ArgLevel) -> LevelFilter {
        match value {
            ArgLevel::Trace => LevelFilter::TRACE,
            ArgLevel::Debug => LevelFilter::DEBUG,
            ArgLevel::Info => LevelFilter::INFO,
            ArgLevel::Warn => LevelFilter::WARN,
            ArgLevel::Error => LevelFilter::ERROR,
            ArgLevel::Off => LevelFilter::OFF,
        }
    }
}

#[derive(Debug, Args)]
struct CommonArgs {
    /// input fst file
    input_file: PathBuf,
    /// output format
    #[arg(short, long, value_enum, default_value_t)]
    format: OutputFormat,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows all blocks in a FST file.
    List {
        #[command(flatten)]
        common: CommonArgs,
    },
    Show {
        #[command(flatten)]
        common: CommonArgs,
        /// the index of the block to show
        block_index: usize,
    },
    /// Dump the contents of a block to a file.
    /// If the block data is compressed, it will first uncompress the data and dump the contents.
    DumpData {
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
    /// Counts blocks in FST file and displays it.
    Stats {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Counts blocks in FST file and displays it.
    Header {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Shows Hierarchy
    Hierarchy {
        #[command(flatten)]
        common: CommonArgs,
        /// show only tokens
        #[arg(short, long, default_value_t = false)]
        show_tokens: bool,
    },
    /// Shows Geometry
    Geometry {
        #[command(flatten)]
        common: CommonArgs,
    },

    /// Shows Geometry
    Blackout {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Shows Value Cahnge Data
    Vcd {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(short, long)]
        intermediate: bool,
    },
}

impl CliArgs {
    fn get_common(&self) -> &CommonArgs {
        match &self.command {
            Commands::List { common, .. } => common,
            Commands::Show { common, .. } => common,
            Commands::DumpData { common, .. } => common,
            Commands::DumpAll { common } => common,
            Commands::Stats { common } => common,
            Commands::Header { common, .. } => common,
            Commands::Hierarchy { common, .. } => common,
            Commands::Geometry { common, .. } => common,
            Commands::Blackout { common, .. } => common,
            Commands::Vcd { common, .. } => common,
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

static IS_TERMINAL: OnceLock<bool> = OnceLock::new();

fn is_terminal() -> &'static bool {
    IS_TERMINAL.get_or_init(|| std::io::stdout().is_terminal())
}
trait OnlyOnTerminal: Sized + fmt::Display {
    fn only_on_terminal(&self) -> Cow<'static, str> {
        if *is_terminal() {
            Cow::Owned(format!("{}", self))
        } else {
            Cow::Borrowed("")
        }
    }
}

impl<T> OnlyOnTerminal for T where T: fmt::Display {}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = CliArgs::parse();
    tracing_subscriber::registry()
        .with(LevelFilter::from(args.log_level))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
        )
        .with(tracing_error::ErrorLayer::default())
        .init();

    trace!("start of cli");
    debug!("cli arguments {args:?}");

    let common = args.get_common();

    let mut file = File::open(&common.input_file)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    match args.command {
        Commands::List {
            common: CommonArgs { format, .. },
            ..
        } => {
            let blocks = fst_file::parse_raw_block_information(&contents).unwrap();
            match format {
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
            }
        }
        Commands::Show { .. } => todo!(),
        Commands::DumpData {
            block_index,
            output_file: output,
            ..
        } => {
            let blocks = fst_file::parse_raw_block_information(&contents).unwrap();
            let mut output_file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(output)
                .unwrap();
            let block_info = blocks.get(block_index).unwrap();
            output_file
                .write_all(&block_info.get_block().extract_data()?)
                .unwrap();
        }
        Commands::DumpAll { .. } => todo!(),
        Commands::Stats { .. } => {
            let blocks = fst_file::parse_raw_block_information(&contents).unwrap();
            let mut data = HashMap::new();
            for block in blocks.iter() {
                let entry = data.entry(block.get_block().block_type).or_insert(0);
                *entry += 1;
            }
            let mut v: Vec<_> = data.into_iter().collect();
            v.sort_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2));
            let width = v.iter().map(|(k, _v)| k.to_string().len()).max().unwrap();
            let width = width.max(10);
            println!("{type_text:>width$} count", type_text = "block type",);
            for (block_type, num) in v {
                println!(
                    "{bold}{block_type:>width$}{reset_style} {green}{num}{reset_color}",
                    bold = termion::style::Bold.only_on_terminal(),
                    reset_style = termion::style::Reset.only_on_terminal(),
                    green = color::Fg(color::Green).only_on_terminal(),
                    reset_color = color::Fg(color::Reset).only_on_terminal()
                );
            }
        }
        Commands::Header {
            common: CommonArgs { format, .. },
            ..
        } => {
            let blocks = fst_file::parse(&contents).unwrap();
            let header_block = blocks.header.unwrap();
            match header_block.get_content() {
                Ok(content) => match format {
                    OutputFormat::PlainText => println!("{:#?}", content),

                    OutputFormat::Json => {
                        print!("{}", serde_json::to_string(&content).unwrap())
                    }
                    OutputFormat::PrettyJson => {
                        println!("{}", serde_json::to_string_pretty(&content).unwrap())
                    }
                },
                Err(e) => {
                    error!("Error while parsing header content {:?}", e)
                }
            }
        }
        Commands::Hierarchy {
            common: CommonArgs { format, .. },
            show_tokens,
            ..
        } => {
            let blocks = fst_file::parse(&contents).unwrap();
            if let Some(hierarchy_block) = blocks.hierarchy {
                if show_tokens {
                    match hierarchy_block.get_tokens() {
                        Ok(content) => {
                            let _span = debug_span!("printing hierarchy tokens");
                            match format {
                                OutputFormat::PlainText => {
                                    for (idx, (s, t)) in content.iter().enumerate() {
                                        if s.size == 1 {
                                            println!("#{idx} [{}] {:#?}", s.position, t)
                                        } else {
                                            println!(
                                                "#{idx} [{}..{}] {:#?}",
                                                s.position,
                                                s.position + s.size - 1,
                                                t
                                            )
                                        }
                                    }
                                }
                                OutputFormat::Json => {
                                    print!("{}", serde_json::to_string(&content).unwrap())
                                }
                                OutputFormat::PrettyJson => {
                                    println!("{}", serde_json::to_string_pretty(&content).unwrap())
                                }
                            }
                        }
                        Err(e) => error!("Error while parsing header content {:?}", e),
                    }
                } else {
                    match hierarchy_block.get_content() {
                        Ok(hierarchy) => {
                            let _span = debug_span!("printing hierarchy");
                            match format {
                                OutputFormat::PlainText => {
                                    println!("{hierarchy:#?}")
                                }
                                OutputFormat::Json => {
                                    print!("{}", serde_json::to_string(&hierarchy).unwrap())
                                }
                                OutputFormat::PrettyJson => {
                                    println!(
                                        "{}",
                                        serde_json::to_string_pretty(&hierarchy).unwrap()
                                    )
                                }
                            }
                        }
                        Err(e) => error!("Error while parsing header content {:?}", e),
                    }
                }
            } else {
                error!("Hierarchy block did not exist in file!");
            }
        }
        Commands::Geometry { .. } => {
            let blocks = fst_file::parse(&contents).unwrap();
            let Some(hierarchy_block) = blocks.geometry else {
                panic!("Geometry block did not exist in file!");
            };
            match hierarchy_block.get_content() {
                Ok(geom) => println!("{:?}", geom),
                Err(e) => error!("Error while parsing header content {:?}", e),
            }
        }
        Commands::Blackout { .. } => {
            let blocks = fst_file::parse(&contents).unwrap();
            let Some(blackout_block) = blocks.blackout else {
                panic!("Geometry block did not exist in file!");
            };
            match blackout_block.get_content() {
                Ok(content) => println!("{:?}", content),
                Err(e) => error!("Error while parsing header content {:?}", e),
            }
        }
        Commands::Vcd { intermediate, .. } => {
            let blocks = fst_file::parse(&contents).unwrap();
            let header_content = blocks.header.unwrap().get_content().unwrap();
            for vcd_block in blocks.value_change_data.iter() {
                if intermediate {
                    match vcd_block.get_intermediate_content(&header_content) {
                        Ok(vcd) => println!("{:?}", vcd),
                        Err(e) => println!("Error! {}", e),
                    }
                } else {
                    match vcd_block.get_content(&header_content) {
                        Ok(vcd) => println!("{:?}", vcd),
                        Err(e) => println!("Error! {}", e),
                    }
                }
            }
        }
    }
    Ok(())
}
