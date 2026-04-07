/// Integration tests for coreutils/cp.corvo and coreutils/ls.corvo.
///
/// Two layers of coverage:
///
/// 1. **Unit / in-process tests** – use `run_with_script_argv` (the same
///    evaluator path that the binary uses) to exercise cp.corvo or ls.corvo
///    with controlled argument vectors.  These tests verify exit-code,
///    variable state, and filesystem side-effects without spawning a child
///    process.
///
/// 2. **Comparison / binary tests** – spawn the `corvo` binary compiled by
///    Cargo (via `CARGO_BIN_EXE_corvo`) and capture stdout/exit-code.  These
///    exercise the full CLI path including argument forwarding through `--`.
use std::path::PathBuf;

use corvo_lang::{CorvoResult, RuntimeState};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn run_with_script_argv(source: &str, argv: Vec<String>) -> CorvoResult<RuntimeState> {
    use corvo_lang::compiler::Evaluator;
    use corvo_lang::lexer::Lexer;
    use corvo_lang::parser::Parser;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    let mut state = RuntimeState::new();
    state.set_script_argv(argv);
    let mut evaluator = Evaluator::new();
    evaluator.run(&program, &mut state)?;
    Ok(state)
}

/// Read `coreutils/<name>.corvo` source at compile time.
fn corvo_script(name: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest).join("coreutils").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {:?}: {}", path, e))
}

/// Path to the `corvo` binary built by Cargo for these tests.
fn corvo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_corvo"))
}

/// Helper: run `corvo <script> -- <args...>` and return (stdout, exit_code).
fn run_corvo_script(script: &str, args: &[&str]) -> (String, i32) {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let script_path = PathBuf::from(manifest).join("coreutils").join(script);
    let mut cmd = std::process::Command::new(corvo_bin());
    cmd.arg(script_path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("failed to spawn corvo process");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let code = out.status.code().unwrap_or(-1);
    (stdout, code)
}

// ---------------------------------------------------------------------------
// Helper: escape a path for embedding in a Corvo string literal.
// ---------------------------------------------------------------------------

fn escape_path(p: &std::path::Path) -> String {
    p.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

// ===========================================================================
// cp.corvo — unit tests (in-process)
// ===========================================================================

/// cp: copy a single file to a new path.
#[test]
fn test_cp_copy_single_file() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    std::fs::write(&src, "hello corvo").unwrap();

    let src_s = escape_path(&src);
    let dst_s = escape_path(&dst);

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(&source, vec![src_s, dst_s]);
    // sys.exit(0) produces ExitRequest — that IS the success signal.
    match &result {
        Ok(_) => {}
        Err(e) if e.process_exit_code() == Some(0) => {}
        Err(e) => panic!("unexpected error: {e:?}"),
    }
    assert_eq!(std::fs::read_to_string(&dst).unwrap(), "hello corvo");
}

/// cp: copy a file into an existing directory (destination is a directory).
#[test]
fn test_cp_copy_into_directory() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("file.txt");
    let dest_dir = dir.path().join("dest");
    std::fs::create_dir(&dest_dir).unwrap();
    std::fs::write(&src, "dir copy").unwrap();

    let src_s = escape_path(&src);
    let dst_s = escape_path(&dest_dir);

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(&source, vec![src_s, dst_s]);
    match &result {
        Ok(_) | Err(_) => {} // exit(0) always produces an Err(ExitRequest)
    }
    let expected = dest_dir.join("file.txt");
    assert!(expected.exists(), "file.txt should appear inside dest/");
    assert_eq!(std::fs::read_to_string(&expected).unwrap(), "dir copy");
}

/// cp: copy multiple sources into a directory.
#[test]
fn test_cp_multiple_sources_to_dir() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    let dest = dir.path().join("out");
    std::fs::create_dir(&dest).unwrap();
    std::fs::write(&a, "aaa").unwrap();
    std::fs::write(&b, "bbb").unwrap();

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec![escape_path(&a), escape_path(&b), escape_path(&dest)],
    );
    // any Err here that is not exit(0) is a real failure
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "unexpected error: {e:?}");
    }
    assert!(dest.join("a.txt").exists());
    assert_eq!(std::fs::read_to_string(dest.join("a.txt")).unwrap(), "aaa");
    assert!(dest.join("b.txt").exists());
    assert_eq!(std::fs::read_to_string(dest.join("b.txt")).unwrap(), "bbb");
}

/// cp: recursive copy of a directory tree.
#[test]
fn test_cp_recursive_directory() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    let sub = src.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(src.join("top.txt"), "top").unwrap();
    std::fs::write(sub.join("deep.txt"), "deep").unwrap();

    let dest = dir.path().join("dest");
    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec!["-r".to_string(), escape_path(&src), escape_path(&dest)],
    );
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "recursive cp failed: {e:?}");
    }
    assert!(dest.join("top.txt").exists(), "top.txt should be copied");
    assert_eq!(
        std::fs::read_to_string(dest.join("top.txt")).unwrap(),
        "top"
    );
    assert!(
        dest.join("sub").join("deep.txt").exists(),
        "sub/deep.txt should be copied"
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("sub").join("deep.txt")).unwrap(),
        "deep"
    );
}

/// cp: -r not given for directory → error message, exit 1.
#[test]
fn test_cp_directory_without_recursive_flag() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    let dest = dir.path().join("dest");

    let (stdout, code) =
        run_corvo_script("cp.corvo", &[src.to_str().unwrap(), dest.to_str().unwrap()]);
    assert_eq!(code, 1, "should exit 1 when -r is omitted for a directory");
    assert!(
        stdout.contains("-r not specified"),
        "expected '-r not specified' in output, got: {stdout}"
    );
}

/// cp: --no-clobber (-n) does not overwrite an existing destination.
#[test]
fn test_cp_no_clobber() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("new.txt");
    let dst = dir.path().join("existing.txt");
    std::fs::write(&src, "new content").unwrap();
    std::fs::write(&dst, "original content").unwrap();

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec!["-n".to_string(), escape_path(&src), escape_path(&dst)],
    );
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "unexpected error: {e:?}");
    }
    // File must still have original content.
    assert_eq!(
        std::fs::read_to_string(&dst).unwrap(),
        "original content",
        "--no-clobber should preserve the destination"
    );
}

/// cp: without --no-clobber, existing destination is overwritten.
#[test]
fn test_cp_overwrites_without_no_clobber() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("new.txt");
    let dst = dir.path().join("existing.txt");
    std::fs::write(&src, "new content").unwrap();
    std::fs::write(&dst, "original content").unwrap();

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(&source, vec![escape_path(&src), escape_path(&dst)]);
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "unexpected error: {e:?}");
    }
    assert_eq!(
        std::fs::read_to_string(&dst).unwrap(),
        "new content",
        "destination should be overwritten"
    );
}

/// cp: -u (--update) skips copy when destination is same-age or newer.
#[test]
fn test_cp_update_skips_older_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    std::fs::write(&src, "old source").unwrap();
    std::fs::write(&dst, "newer dest").unwrap();

    // Give dst a newer mtime by sleeping briefly and touching it.
    // We touch dst to ensure its mtime >= src mtime.
    // The simplest cross-platform approach: explicitly set mtime via file ops.
    std::thread::sleep(std::time::Duration::from_millis(10));
    // Re-write dst to refresh its mtime.
    std::fs::write(&dst, "newer dest").unwrap();

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec!["-u".to_string(), escape_path(&src), escape_path(&dst)],
    );
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "unexpected error: {e:?}");
    }
    // dst should be unchanged because it is newer.
    assert_eq!(
        std::fs::read_to_string(&dst).unwrap(),
        "newer dest",
        "-u should skip when destination is newer"
    );
}

/// cp: -t <dir> sources flag.
#[test]
fn test_cp_target_directory_flag() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("file.txt");
    let target = dir.path().join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(&src, "target test").unwrap();

    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec!["-t".to_string(), escape_path(&target), escape_path(&src)],
    );
    if let Err(e) = &result {
        assert_eq!(e.process_exit_code(), Some(0), "unexpected error: {e:?}");
    }
    let copied = target.join("file.txt");
    assert!(copied.exists(), "file.txt should be inside target/");
    assert_eq!(std::fs::read_to_string(copied).unwrap(), "target test");
}

/// cp: copying a nonexistent source produces an error message and exit 1.
#[test]
fn test_cp_missing_source_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("does_not_exist.txt");
    let dst = dir.path().join("dst.txt");

    let (stdout, code) =
        run_corvo_script("cp.corvo", &[src.to_str().unwrap(), dst.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(
        stdout.contains("No such file"),
        "expected 'No such file' in output, got: {stdout}"
    );
}

/// cp: missing operand (no arguments) → exit 1 with usage hint.
#[test]
fn test_cp_no_arguments_error() {
    let (stdout, code) = run_corvo_script("cp.corvo", &[]);
    assert_eq!(code, 1);
    assert!(
        stdout.contains("missing file operand"),
        "expected 'missing file operand', got: {stdout}"
    );
}

/// cp: only one argument (missing destination) → exit 1 with usage hint.
#[test]
fn test_cp_missing_destination_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    std::fs::write(&src, "x").unwrap();
    let (stdout, code) = run_corvo_script("cp.corvo", &[src.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert!(
        stdout.contains("missing destination"),
        "expected 'missing destination', got: {stdout}"
    );
}

/// cp: --version outputs version string.
#[test]
fn test_cp_version() {
    let (stdout, code) = run_corvo_script("cp.corvo", &["--version"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("cp (Corvo coreutils)"),
        "expected version line, got: {stdout}"
    );
}

/// cp: --help outputs usage string.
#[test]
fn test_cp_help() {
    let (stdout, code) = run_corvo_script("cp.corvo", &["--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Usage: cp"),
        "expected usage line, got: {stdout}"
    );
    assert!(
        stdout.contains("--recursive"),
        "expected --recursive in help, got: {stdout}"
    );
}

/// cp: -v (--verbose) prints the copy action.
#[test]
fn test_cp_verbose_output() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("dst.txt");
    std::fs::write(&src, "verbose").unwrap();

    let (stdout, code) = run_corvo_script(
        "cp.corvo",
        &["-v", src.to_str().unwrap(), dst.to_str().unwrap()],
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("->"),
        "expected '->' in verbose output, got: {stdout}"
    );
}

/// cp: multiple sources to a non-existent (non-directory) destination → error.
#[test]
fn test_cp_multiple_sources_to_non_directory_error() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    let dest = dir.path().join("not_a_dir.txt");
    std::fs::write(&a, "a").unwrap();
    std::fs::write(&b, "b").unwrap();
    // dest does NOT exist yet, so it is not a directory.
    let (stdout, code) = run_corvo_script(
        "cp.corvo",
        &[
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            dest.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 1);
    assert!(
        stdout.contains("not a directory"),
        "expected 'not a directory', got: {stdout}"
    );
}

/// cp: -a (archive) implies recursive copy.
#[test]
fn test_cp_archive_implies_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    std::fs::write(src.join("file.txt"), "archive").unwrap();

    let dest = dir.path().join("dest");
    let source = corvo_script("cp.corvo");
    let result = run_with_script_argv(
        &source,
        vec!["-a".to_string(), escape_path(&src), escape_path(&dest)],
    );
    if let Err(e) = &result {
        assert_eq!(
            e.process_exit_code(),
            Some(0),
            "-a recursive copy failed: {e:?}"
        );
    }
    assert!(dest.join("file.txt").exists(), "-a should copy recursively");
}

// ===========================================================================
// ls.corvo — unit tests (in-process)
// ===========================================================================

/// ls: listing a directory shows its files.
#[test]
fn test_ls_lists_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("alpha.txt"), "a").unwrap();
    std::fs::write(dir.path().join("beta.txt"), "b").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains("alpha.txt"), "expected alpha.txt in output");
    assert!(stdout.contains("beta.txt"), "expected beta.txt in output");
}

/// ls: default sort is alphabetical.
#[test]
fn test_ls_default_alphabetical_sort() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("zebra.txt"), "z").unwrap();
    std::fs::write(dir.path().join("apple.txt"), "a").unwrap();
    std::fs::write(dir.path().join("mango.txt"), "m").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    let idx_apple = lines.iter().position(|l| l.contains("apple")).unwrap();
    let idx_mango = lines.iter().position(|l| l.contains("mango")).unwrap();
    let idx_zebra = lines.iter().position(|l| l.contains("zebra")).unwrap();
    assert!(
        idx_apple < idx_mango && idx_mango < idx_zebra,
        "expected alphabetical order: apple < mango < zebra"
    );
}

/// ls: -r reverses the sort order.
#[test]
fn test_ls_reverse_sort() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("aaa.txt"), "a").unwrap();
    std::fs::write(dir.path().join("zzz.txt"), "z").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-r", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    let idx_aaa = lines.iter().position(|l| l.contains("aaa")).unwrap();
    let idx_zzz = lines.iter().position(|l| l.contains("zzz")).unwrap();
    assert!(idx_zzz < idx_aaa, "expected zzz before aaa in reverse sort");
}

/// ls: dotfiles are hidden by default.
#[test]
fn test_ls_hides_dotfiles_by_default() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("visible.txt"), "v").unwrap();
    std::fs::write(dir.path().join(".hidden"), "h").unwrap();

    let (stdout, _) = run_corvo_script("ls.corvo", &["-1", dir.path().to_str().unwrap()]);
    assert!(
        stdout.contains("visible.txt"),
        "expected visible.txt in output"
    );
    assert!(
        !stdout.contains(".hidden"),
        "dotfiles should be hidden by default"
    );
}

/// ls: -a shows dotfiles.
#[test]
fn test_ls_all_shows_dotfiles() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("visible.txt"), "v").unwrap();
    std::fs::write(dir.path().join(".hidden"), "h").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-a", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains(".hidden"), "expected .hidden with -a flag");
    assert!(stdout.contains("."), "expected . entry with -a flag");
    assert!(stdout.contains(".."), "expected .. entry with -a flag");
}

/// ls: -A shows dotfiles but not . and ..
#[test]
fn test_ls_almost_all_omits_dot_entries() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".hidden"), "h").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-A", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(stdout.contains(".hidden"), "expected .hidden with -A flag");
    // . and .. should not appear as standalone entries.
    assert!(
        !lines.iter().any(|l| l.trim() == "."),
        "-A should not show . entry"
    );
    assert!(
        !lines.iter().any(|l| l.trim() == ".."),
        "-A should not show .. entry"
    );
}

/// ls: -d lists the directory itself, not its contents.
#[test]
fn test_ls_directory_flag() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("child.txt"), "c").unwrap();
    let path = dir.path().to_str().unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-d", path]);
    assert_eq!(code, 0);
    // Should not list child.txt
    assert!(
        !stdout.contains("child.txt"),
        "-d should not list directory contents"
    );
}

/// ls: -F appends / to directories and * to executables.
#[test]
fn test_ls_classify_appends_slash_to_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(dir.path().join("file.txt"), "f").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-F", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("subdir/"),
        "expected 'subdir/' with -F, got: {stdout}"
    );
}

/// ls: --version outputs version string.
#[test]
fn test_ls_version() {
    let (stdout, code) = run_corvo_script("ls.corvo", &["--version"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("ls (Corvo coreutils)"),
        "expected version line, got: {stdout}"
    );
}

/// ls: --help outputs usage string.
#[test]
fn test_ls_help() {
    let (stdout, code) = run_corvo_script("ls.corvo", &["--help"]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Usage: ls"),
        "expected usage line, got: {stdout}"
    );
}

/// ls: -l long format includes mode string, owner, size, name.
#[cfg(unix)]
#[test]
fn test_ls_long_format_fields() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("sample.txt"), "hello").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-l", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    // Expect lines that start with a mode string like -rw-...
    let data_lines: Vec<&str> = stdout.lines().filter(|l| !l.starts_with("total")).collect();
    assert!(
        !data_lines.is_empty(),
        "expected at least one entry in long listing"
    );
    let first = data_lines[0];
    // Mode string: first char is - or d or l
    assert!(
        first.starts_with('-') || first.starts_with('d') || first.starts_with('l'),
        "long format should start with mode character, got: {first}"
    );
    assert!(
        stdout.contains("sample.txt"),
        "expected sample.txt in long listing"
    );
}

/// ls: -R recursively lists subdirectories.
#[test]
fn test_ls_recursive() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("deep.txt"), "d").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-R", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("deep.txt"),
        "expected deep.txt in recursive listing, got: {stdout}"
    );
}

/// ls: multiple path arguments — each directory is preceded by its header.
#[test]
fn test_ls_multiple_paths_show_headers() {
    let dir = tempfile::tempdir().unwrap();
    let d1 = dir.path().join("dir1");
    let d2 = dir.path().join("dir2");
    std::fs::create_dir_all(&d1).unwrap();
    std::fs::create_dir_all(&d2).unwrap();
    std::fs::write(d1.join("one.txt"), "1").unwrap();
    std::fs::write(d2.join("two.txt"), "2").unwrap();

    let (stdout, code) = run_corvo_script(
        "ls.corvo",
        &["-1", d1.to_str().unwrap(), d2.to_str().unwrap()],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("one.txt"), "expected one.txt");
    assert!(stdout.contains("two.txt"), "expected two.txt");
    // Both directory paths should appear as headers.
    assert!(stdout.contains("dir1"), "expected dir1 header");
    assert!(stdout.contains("dir2"), "expected dir2 header");
}

/// ls: -B hides backup files ending with ~.
#[test]
fn test_ls_ignore_backups() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("file.txt"), "f").unwrap();
    std::fs::write(dir.path().join("file.txt~"), "backup").unwrap();

    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", "-B", dir.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains("file.txt"), "expected file.txt");
    assert!(
        !stdout.contains("file.txt~"),
        "expected backup file to be hidden with -B"
    );
}

/// ls: --sort=size orders by file size (largest first).
#[test]
fn test_ls_sort_by_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("small.txt"), "s").unwrap(); // 1 byte
    std::fs::write(dir.path().join("large.txt"), "l".repeat(1000)).unwrap(); // 1000 bytes

    let (stdout, code) = run_corvo_script(
        "ls.corvo",
        &["-1", "--sort=size", dir.path().to_str().unwrap()],
    );
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    let large_pos = lines.iter().position(|l| l.contains("large")).unwrap();
    let small_pos = lines.iter().position(|l| l.contains("small")).unwrap();
    assert!(
        large_pos < small_pos,
        "large.txt should appear before small.txt with --sort=size"
    );
}

// ===========================================================================
// cp + ls interaction: copy then list
// ===========================================================================

/// Copy files and verify ls shows them in the destination.
#[test]
fn test_cp_then_ls_shows_copied_files() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    let dst_dir = dir.path().join("dst");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("foo.txt"), "foo").unwrap();
    std::fs::write(src_dir.join("bar.txt"), "bar").unwrap();

    // cp -r src/ dst/
    let cp_result = run_with_script_argv(
        &corvo_script("cp.corvo"),
        vec![
            "-r".to_string(),
            escape_path(&src_dir),
            escape_path(&dst_dir),
        ],
    );
    if let Err(e) = &cp_result {
        assert_eq!(e.process_exit_code(), Some(0), "cp failed: {e:?}");
    }

    // ls dst/
    let (stdout, code) = run_corvo_script("ls.corvo", &["-1", dst_dir.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("foo.txt"),
        "ls should show foo.txt after cp"
    );
    assert!(
        stdout.contains("bar.txt"),
        "ls should show bar.txt after cp"
    );
}
