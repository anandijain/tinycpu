use std::{fs, io, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use tinycpu::{assembler::assemble, compiler, memory::save_memory, rom::create_rom};

#[derive(Parser)]
#[command(
    name = "tinycpu",
    version,
    about = "Compile tinycpu programs or generate the control ROM"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile assembly into a RAM image. Does not generate or write the control ROM.
    Assemble {
        /// Assembly source, with optional NAME = ADDRESS definitions
        source: PathBuf,
        /// RAM image to write, relative to the current directory
        #[arg(short, long, default_value = "RAM.txt")]
        output: PathBuf,
    },
    /// Compile the higher-level language (not implemented yet)
    Compile {
        /// Higher-level source program
        source: PathBuf,
        /// RAM image to write, relative to the current directory
        #[arg(short, long, default_value = "RAM.txt")]
        output: PathBuf,
    },
    /// Generate the control ROM after changing the CPU's microcode
    Rom {
        /// ROM image to write, relative to the current directory
        #[arg(short, long, default_value = "ROM.txt")]
        output: PathBuf,
    },
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Assemble { source, output } => {
            let text = fs::read_to_string(&source)
                .map_err(|error| format!("could not read {}: {error}", source.display()))?;
            let ram = assemble(&text).map_err(|error| format!("{}: {error}", source.display()))?;
            save_memory(&output, &ram, 2, 2).map_err(|error| output_error(&output, error))?;
            println!("Wrote RAM image to {}", output.display());
        }
        Command::Rom { output } => {
            let rom = create_rom();
            save_memory(&output, &rom, 2, 4).map_err(|error| output_error(&output, error))?;
            println!("Wrote control ROM to {}", output.display());
        }
        Command::Compile { source, output } => {
            let text = fs::read_to_string(&source)
                .map_err(|error| format!("could not read {}: {error}", source.display()))?;
            let ram = compiler::compile(&text).unwrap();
            save_memory(&output, &ram, 2, 2).map_err(|error| output_error(&output, error))?;
            println!("Wrote RAM image to {}", output.display());
        }
    }
    Ok(())
}

fn output_error(path: &std::path::Path, error: io::Error) -> String {
    format!("could not write {}: {error}", path.display())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
