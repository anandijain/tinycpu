# tinycpu tools

The `tinycpu` CLI has independent commands for program compilation and control-ROM
writing. Compiling programs does not regenerate or overwrite the control ROM.

From the repository root:

```sh
# Compile a source program and keep both generated levels beside it.
cargo run --release -- compile write_rom2/programs/1/1.aj

# Assemble instructions into RAM.txt in the current directory.
cargo run -- assemble write_rom2/programs/old/symbolic_add.txt

# Choose a different RAM output. Missing parent directories are created.
cargo run -- assemble write_rom2/programs/old/add_2_numbers.s -o build/RAM.txt

# Only run this when the CPU's microcode changes.
cargo run -- rom -o ROM.txt

cargo run -- --help
cargo run -- compile --help
cargo run -- assemble --help
cargo run -- rom --help
```

`compile` calls the higher-level compiler, saves its symbolic assembly, then feeds
that assembly to `assemble` and saves the resulting Logisim RAM image:

```text
programs/
  1/
    1.aj      source
    1.sasm    generated symbolic assembly
    1.ram     generated v3.0 hex words addressed RAM image
```

From inside `write_rom2`, run `cargo run --release -- compile programs/1/1.aj`.
Output names use the source's filename with its extension replaced, so they stay
in the source's folder regardless of the current directory. `compile -o PATH`
overrides only the RAM destination; the `.sasm` file stays beside the source.

`assemble` handles existing assembly programs and defaults to `RAM.txt`. `rom`
defaults to `ROM.txt` and does not require a source. Both accept `-o` / `--output`.
Explicit source and output paths are relative to the current directory.

Compiler errors leave existing outputs untouched. If assembling generated code
fails, the `.sasm` file remains available for debugging and the existing `.ram`
file is left untouched. Errors identify the relevant source or output path;
assembler errors also include line numbers. Failed commands exit unsuccessfully.

Install once to call the tool directly (reinstall after changing its Rust code):

```sh
cargo install --path write_rom2 --locked
tinycpu compile write_rom2/programs/1/1.aj
tinycpu assemble write_rom2/programs/old/symbolic_add.txt -o RAM.txt
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
12-bit addresses; 16-bit data values occupy two bytes.

The higher-level compiler currently emits symbolic assembly for top-level integer
literals. For example, `1.aj` containing `1` emits a temporary address definition,
`LDI 1`, and a `STA` to that address. Other parsed forms are currently ignored;
assignments, arithmetic expressions, and control flow remain future work.

The existing microcode is preserved. In particular, `SUB` still has placeholder
NOP microcode; accepting the assembly mnemonic does not implement subtraction
in the hardware.

## Verification

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
```

The tests cover numeric and symbolic operands, invalid input, RAM capacity,
the source-to-symbolic-assembly-to-RAM pipeline, independent CLI outputs, and
equality with the checked-in control-ROM image.
