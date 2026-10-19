# tinycpu tools

The `tinycpu` CLI has independent commands for program compilation and control-ROM
writing. Compiling programs does not regenerate or overwrite the control ROM.

From the repository root:

```sh
# Assemble instructions into RAM.txt in the current directory.
cargo run -- assemble write_rom2/programs/symbolic_add.txt

# Choose a different RAM output. Missing parent directories are created.
cargo run -- assemble write_rom2/programs/add_2_numbers.txt -o build/RAM.txt

# Only run this when the CPU's microcode changes.
cargo run -- rom -o ROM.txt

cargo run -- --help
cargo run -- assemble --help
cargo run -- rom --help
```

`assemble` handles assembly programs. `compile` is reserved for the higher-level
language being built in `compiler.rs`; it currently returns an error without
writing an image. Both commands accept `-o` / `--output`.
The defaults are `RAM.txt` and `ROM.txt`, relative to the current working directory.
Each command writes exactly one image; there are no implicit duplicate writes.
Source paths are also relative to the current directory. Compilation errors include
source filenames and line numbers, exit unsuccessfully, and leave existing output
untouched. The ROM command does not require a source program.

Install once to call the tool directly (reinstall after changing its Rust code):

```sh
cargo install --path write_rom2 --locked
tinycpu assemble write_rom2/programs/symbolic_add.txt -o RAM.txt
tinycpu rom -o ROM.txt
```

## Code layout

| File | Responsibility |
| --- | --- |
| `src/isa.rs` | Shared `Op`, `Instruction`, encoding, operand limits, and RAM dimensions |
| `src/assembler.rs` | Parse assembly and symbol definitions; produce RAM bytes |
| `src/compiler.rs` | Higher-level language compiler, being rebuilt incrementally |
| `src/rom.rs` | Control signals, bus sources, microinstruction sequences, and control-ROM generation |
| `src/memory.rs` | Format and save Logisim memory images |
| `src/main.rs` | Parse CLI arguments and run the selected operation |
| `src/lib.rs` | Expose modules for reuse |

Both the compiler and ROM generator depend on `isa`; the compiler does not depend
on ROM generation. File I/O happens outside the compiler. All tools remain in one
Rust crate, with a workspace manifest and lockfile at the repository root.

## Current source language

The assembler currently accepts assembly, decimal operands, semicolon comments,
and named numeric addresses defined before use:

```text
x = 4094
LDI 5
STA x
HLT
```

Here `x = 4094` is an assembler symbol definition, not a runtime assignment.
Instructions occupy two bytes in big-endian order. RAM is byte-addressed with
12-bit addresses; 16-bit data values occupy two bytes. High-level assignments,
expressions, and control flow in `compiler_levels/` remain future work.

The existing microcode is preserved. In particular, `SUB` still has placeholder
NOP microcode; accepting the assembly mnemonic does not implement subtraction
in the hardware.

## Verification

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
```

The tests cover numeric and symbolic operands, invalid input, RAM capacity,
independent CLI outputs, and equality with the checked-in control-ROM image.
