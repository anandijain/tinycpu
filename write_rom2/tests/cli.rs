use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("tinycpu-cli-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tinycpu"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_keeps_ram_and_rom_generation_independent() {
    let dir = TestDir::new();
    fs::write(dir.0.join("program.asm"), "x = 4094\nLDI 5\nSTA x\nHLT\n").unwrap();
    fs::write(dir.0.join("ROM.txt"), "existing control ROM").unwrap();

    let assembled = dir.run(&["assemble", "program.asm"]);
    assert!(
        assembled.status.success(),
        "{}",
        String::from_utf8_lossy(&assembled.stderr)
    );
    let ram = fs::read(dir.0.join("RAM.txt")).unwrap();
    assert!(String::from_utf8_lossy(&ram).contains("00: 50 05 4f fe f0 00"));
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        "existing control ROM"
    );

    let custom = dir.run(&["assemble", "program.asm", "-o", "nested/custom-ram.txt"]);
    assert!(custom.status.success());
    assert_eq!(fs::read(dir.0.join("nested/custom-ram.txt")).unwrap(), ram);

    fs::write(dir.0.join("bad.asm"), "LDA undefined\n").unwrap();
    let failed = dir.run(&["assemble", "bad.asm"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("bad.asm: line 1:"));
    assert_eq!(fs::read(dir.0.join("RAM.txt")).unwrap(), ram);
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        "existing control ROM"
    );

    let rom = dir.run(&["rom", "--output", "nested/custom-rom.txt"]);
    assert!(
        rom.status.success(),
        "{}",
        String::from_utf8_lossy(&rom.stderr)
    );
    assert_eq!(
        fs::read_to_string(dir.0.join("nested/custom-rom.txt")).unwrap(),
        include_str!("../../ROM.txt")
    );
    assert_eq!(fs::read(dir.0.join("RAM.txt")).unwrap(), ram);
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        "existing control ROM"
    );

    let default_rom = dir.run(&["rom"]);
    assert!(default_rom.status.success());
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        include_str!("../../ROM.txt")
    );
    assert_eq!(fs::read(dir.0.join("RAM.txt")).unwrap(), ram);
}

#[test]
fn compile_writes_both_lowered_files_beside_the_source() {
    let dir = TestDir::new();
    let program_dir = dir.0.join("programs/my program");
    fs::create_dir_all(&program_dir).unwrap();
    fs::write(program_dir.join("1.aj"), "1\n2\n").unwrap();
    fs::write(dir.0.join("RAM.txt"), "existing RAM").unwrap();
    fs::write(dir.0.join("ROM.txt"), "existing control ROM").unwrap();

    let compiled = dir.run(&["compile", "programs/my program/1.aj"]);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert_eq!(
        fs::read_to_string(program_dir.join("1.sasm")).unwrap(),
        "%0 = 4094\nLDI 1\nSTA %0\n%1 = 4092\nLDI 2\nSTA %1"
    );
    let ram = fs::read_to_string(program_dir.join("1.ram")).unwrap();
    assert!(ram.starts_with("v3.0 hex words addressed\n00: 50 01 4f fe 50 02 4f fc"));
    assert_eq!(
        fs::read_to_string(dir.0.join("RAM.txt")).unwrap(),
        "existing RAM"
    );
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        "existing control ROM"
    );
    assert_eq!(
        fs::read_to_string(program_dir.join("1.aj")).unwrap(),
        "1\n2\n"
    );
}

#[test]
fn compile_supports_an_explicit_ram_output() {
    let dir = TestDir::new();
    fs::write(dir.0.join("program.aj"), "1").unwrap();

    let compiled = dir.run(&["compile", "program.aj", "-o", "build/custom.ram"]);
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(dir.0.join("program.sasm").is_file());
    assert!(dir.0.join("build/custom.ram").is_file());
    assert!(!dir.0.join("program.ram").exists());
}

#[test]
fn compile_reports_errors_and_keeps_ram_when_lowering_fails() {
    let dir = TestDir::new();
    fs::write(dir.0.join("program.sasm"), "existing assembly").unwrap();
    fs::write(dir.0.join("program.ram"), "existing RAM").unwrap();

    // A parser error must not replace either previously generated file.
    fs::write(dir.0.join("program.aj"), "(").unwrap();
    let failed = dir.run(&["compile", "program.aj"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("program.aj: failed to parse"));
    assert_eq!(
        fs::read_to_string(dir.0.join("program.sasm")).unwrap(),
        "existing assembly"
    );
    assert_eq!(
        fs::read_to_string(dir.0.join("program.ram")).unwrap(),
        "existing RAM"
    );

    // This parses as u16 but cannot fit in an LDI's 12-bit operand.
    // Keep the generated assembly for inspection without overwriting good RAM.
    fs::write(dir.0.join("program.aj"), "4096").unwrap();
    let failed = dir.run(&["compile", "program.aj"]);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("program.sasm: line 2:"));
    assert!(
        fs::read_to_string(dir.0.join("program.sasm"))
            .unwrap()
            .contains("LDI 4096")
    );
    assert_eq!(
        fs::read_to_string(dir.0.join("program.ram")).unwrap(),
        "existing RAM"
    );
}

#[test]
fn help_and_invalid_arguments_do_not_create_images() {
    let dir = TestDir::new();
    let help = dir.run(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("assemble") && text.contains("compile") && text.contains("rom"));
    for args in [
        &[][..],
        &["assemble"][..],
        &["compile"][..],
        &["rom", "--output"][..],
        &["unknown"][..],
    ] {
        assert!(!dir.run(args).status.success());
    }
    assert!(!dir.0.join("RAM.txt").exists());
    assert!(!dir.0.join("ROM.txt").exists());
}
