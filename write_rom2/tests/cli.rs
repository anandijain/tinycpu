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

    let compile = dir.run(&["compile", "program.asm"]);
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let ram = fs::read(dir.0.join("RAM.txt")).unwrap();
    assert!(String::from_utf8_lossy(&ram).contains("00: 50 05 4f fe f0 00"));
    assert_eq!(
        fs::read_to_string(dir.0.join("ROM.txt")).unwrap(),
        "existing control ROM"
    );

    let custom = dir.run(&["compile", "program.asm", "-o", "nested/custom-ram.txt"]);
    assert!(custom.status.success());
    assert_eq!(fs::read(dir.0.join("nested/custom-ram.txt")).unwrap(), ram);

    fs::write(dir.0.join("bad.asm"), "LDA undefined\n").unwrap();
    let failed = dir.run(&["compile", "bad.asm"]);
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
fn help_and_invalid_arguments_do_not_create_images() {
    let dir = TestDir::new();
    let help = dir.run(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("compile") && text.contains("rom"));
    for args in [
        &[][..],
        &["compile"][..],
        &["rom", "--output"][..],
        &["unknown"][..],
    ] {
        assert!(!dir.run(args).status.success());
    }
    assert!(!dir.0.join("RAM.txt").exists());
    assert!(!dir.0.join("ROM.txt").exists());
}
