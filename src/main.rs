#![forbid(unsafe_code)]

mod pak;
mod ui;
#[cfg(feature = "gui")]
mod gui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "nepak", version, about = "NewEngine PakBuilder (NEPAK v1)")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Interactive wizard for building a pak (terminal).
    Ui,

    /// Native GUI (egui/eframe).
    #[cfg(feature = "gui")]
    Gui,

    /// Build a .pak from an input directory.
    Build {
        /// Input directory.
        #[arg(long)]
        input: PathBuf,
        /// Output pak file.
        #[arg(long)]
        output: PathBuf,
        /// Optional mount prefix inside pak (e.g. "assets/").
        #[arg(long, default_value = "")]
        prefix: String,
        /// Exclude glob-like substring (repeatable). Simple contains() filter on normalized paths.
        #[arg(long)]
        exclude: Vec<String>,
        /// Use zstd compression for payloads (requires feature "zstd").
        #[arg(long, default_value_t = false)]
        compress: bool,
        /// Zstd level (1..=22). Only used with --compress.
        #[arg(long, default_value_t = 6)]
        zstd_level: i32,
    },

    /// List entries in a pak.
    List {
        #[arg(long)]
        pak: PathBuf,
        /// Print hashes too.
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },

    /// Extract pak to an output directory.
    Extract {
        #[arg(long)]
        pak: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Only extract entries that contain this substring (repeatable).
        #[arg(long)]
        filter: Vec<String>,
    },

    /// Verify pak integrity (hashes, bounds).
    Verify {
        #[arg(long)]
        pak: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let res = match cli.cmd {
        Command::Ui => ui::run(),
        #[cfg(feature = "gui")]
        Command::Gui => gui::run(),
        Command::Build {
            input,
            output,
            prefix,
            exclude,
            compress,
            zstd_level,
        } => pak::build(&input, &output, &prefix, &exclude, compress, zstd_level),
        Command::List { pak, verbose } => pak::list(&pak, verbose),
        Command::Extract { pak, output, filter } => pak::extract(&pak, &output, &filter),
        Command::Verify { pak } => pak::verify(&pak),
    };

    if let Err(e) = res {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}