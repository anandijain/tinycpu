use clap::{Parser, Subcommand};
use std::{fs, io, path::PathBuf, process::ExitCode};
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
    /// Compile a program into symbolic assembly and a Logisim RAM image
    Compile {
        /// Higher-level source program
        source: PathBuf,
        /// RAM output (defaults to the source path with a .ram extension).
        /// Symbolic assembly is always written beside the source as .sasm.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate the control ROM after changing the CPU's microcode
    Rom {
        /// ROM image to write, relative to the current directory
        #[arg(short, long, default_value = "ROM.txt")]
        output: PathBuf,
    },
    CompileSource {
        source: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
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
            let sasm_output = source.with_extension("sasm");
            let output = output.unwrap_or_else(|| source.with_extension("ram"));
            if source == sasm_output || source == output || sasm_output == output {
                return Err(
                    "source, symbolic assembly, and RAM output paths must be different".into(),
                );
            }
            let text = fs::read_to_string(&source)
                .map_err(|error| format!("could not read {}: {error}", source.display()))?;
            let sym_asm = compiler::compile(&text)
                .map_err(|error| format!("{}: {error}", source.display()))?;
            fs::write(&sasm_output, &sym_asm).map_err(|error| output_error(&sasm_output, error))?;
            println!("Wrote symbolic assembly to {}", sasm_output.display());

            let ram = assemble(&sym_asm)
                .map_err(|error| format!("{}: {error}", sasm_output.display()))?;
            save_memory(&output, &ram, 2, 2).map_err(|error| output_error(&output, error))?;
            println!("Wrote RAM image to {}", output.display());
        }
        Command::CompileSource { source, output } => {
            let sasm = compiler::compile(&source)?;
            println!("{}", sasm);
            if let Some(out) = output {
                let ram = assemble(&sasm)?;
                save_memory(&out, &ram, 2, 2).map_err(|error| output_error(&out, error))?;
                println!("Wrote RAM image to {}", out.display());
            }
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
