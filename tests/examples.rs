use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const RUN_EXAMPLES: &[(&str, &[&str])] = &[
    ("crypto.peng", &["sha256:", "aGVsbG8=", "68656c6c6f"]),
    (
        "file_system.peng",
        &["===== FS MODULE TEST =====", "Penguin FS works!"],
    ),
    ("impls.peng", &["i impls int: true", "p impls Person: true"]),
    ("math.peng", &["3.141592653589793", "256", "100"]),
    (
        "os.peng",
        &["===== OS MODULE TEST =====", "hello from os:run"],
    ),
    (
        "sync_atomic.peng",
        &["===== SYNC ATOMIC TEST =====", "2000"],
    ),
    ("sync_channel.peng", &["===== SYNC CHANNEL TEST =====", "6"]),
    ("sync_mutex.peng", &["===== SYNC MUTEX TEST =====", "3000"]),
    ("time.peng", &["===== TIME MODULE TEST =====", "2000"]),
];

fn pabble_bin() -> &'static str {
    env!("CARGO_BIN_EXE_pabble")
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples_dir() -> PathBuf {
    manifest_dir().join("examples")
}

fn example_files() -> Vec<PathBuf> {
    let mut examples = fs::read_dir(examples_dir())
        .expect("failed to read examples directory")
        .map(|entry| entry.expect("failed to read examples entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "peng")
        })
        .collect::<Vec<_>>();

    examples.sort();
    examples
}

fn run_pabble(args: &[&str], cwd: &Path) -> Output {
    Command::new(pabble_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run pabble")
}

fn assert_clean_success(output: &Output) {
    assert!(
        output.status.success() && output.stderr.is_empty(),
        "process failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stdout_contains(output: &Output, expected: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(expected),
        "stdout did not contain {expected:?}\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_stderr_contains(output: &Output, expected: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains(expected),
        "stderr did not contain {expected:?}\nstderr:\n{stderr}\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("pabble-test-{name}-{nonce}"));

    fs::create_dir_all(&path).expect("failed to create temp test directory");

    path
}

#[test]
fn compiles_every_example() {
    let dir = temp_dir("compile-examples");

    for example in example_files() {
        let name = example
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("example has invalid file name");
        let output_path = dir.join(format!("{name}.penb"));
        let example_arg = example.to_string_lossy().into_owned();
        let output_arg = output_path.to_string_lossy().into_owned();
        let output = run_pabble(&["compile", &example_arg, &output_arg], &manifest_dir());

        assert_clean_success(&output);
        assert!(
            output_path.exists(),
            "expected compiled binary for {}",
            example.display()
        );
    }
}

#[test]
fn runs_automatable_examples() {
    for (example, expected_outputs) in RUN_EXAMPLES {
        let path = examples_dir().join(example);
        let path_arg = path.to_string_lossy().into_owned();
        let output = run_pabble(&["run", &path_arg], &manifest_dir());

        assert_clean_success(&output);

        for expected in *expected_outputs {
            assert_stdout_contains(&output, expected);
        }
    }
}

#[test]
fn runs_local_import_program() {
    let dir = temp_dir("import");
    let module_path = dir.join("module.peng");
    let main_path = dir.join("main.peng");

    fs::write(
        &module_path,
        r#"
import("io") as io

func value() {
    return "imported value"
}
"#,
    )
    .expect("failed to write imported module");

    fs::write(
        &main_path,
        r#"
import("io") as io
import("./module.peng") as module

func main() {
    io:println(module:value())
}
"#,
    )
    .expect("failed to write main module");

    let main_arg = main_path.to_string_lossy().into_owned();
    let output = run_pabble(&["run", &main_arg], &dir);

    assert_clean_success(&output);
    assert_stdout_contains(&output, "imported value");
}

#[test]
fn compiles_and_runs_binary_program() {
    let dir = temp_dir("compile");
    let source_path = dir.join("main.peng");
    let binary_path = dir.join("main.penb");

    fs::write(
        &source_path,
        r#"
import("io") as io
import("convert") as convert

func main() {
    var value = convert:to_string(42)
    io:println("compiled:", value)
}
"#,
    )
    .expect("failed to write compile test source");

    let source_arg = source_path.to_string_lossy().into_owned();
    let binary_arg = binary_path.to_string_lossy().into_owned();
    let compile_output = run_pabble(&["compile", &source_arg, &binary_arg], &dir);

    assert_clean_success(&compile_output);
    assert!(
        binary_path.exists(),
        "expected compiled binary to be written"
    );

    let run_output = run_pabble(&["run", &binary_arg], &dir);

    assert_clean_success(&run_output);
    assert_stdout_contains(&run_output, "compiled:");
    assert_stdout_contains(&run_output, "42");
}

#[test]
fn formatted_errors_include_source_paths() {
    let dir = temp_dir("error-path");
    let source_path = dir.join("bad_main.peng");

    fs::write(
        &source_path,
        r#"
import("io") as io

func main() {
    io:println(missing_name)
}
"#,
    )
    .expect("failed to write invalid source");

    let source_arg = source_path.to_string_lossy().into_owned();
    let output = run_pabble(&["run", &source_arg], &dir);

    assert!(
        output.status.success(),
        "pabble should return after printing errors"
    );
    assert_stderr_contains(&output, "bad_main.peng");
    assert_stderr_contains(&output, "line");
    assert_stderr_contains(&output, "unknown value");
}

#[test]
fn formatted_errors_resolve_name_ids() {
    let dir = temp_dir("error-names");
    let no_main_path = dir.join("no_main.peng");
    let missing_attr_path = dir.join("missing_attr.peng");

    fs::write(
        &no_main_path,
        r#"
import("io") as io

func helper() {
    io:println("helper")
}
"#,
    )
    .expect("failed to write no-main source");

    fs::write(
        &missing_attr_path,
        r#"
import("io") as io

func main() {
    var object = {}
    io:println(object.missing_attr)
}
"#,
    )
    .expect("failed to write missing-attribute source");

    let no_main_arg = no_main_path.to_string_lossy().into_owned();
    let no_main_output = run_pabble(&["run", &no_main_arg], &dir);

    assert!(
        no_main_output.status.success(),
        "pabble should return after printing errors"
    );
    assert_stderr_contains(&no_main_output, "name not found: 'main'");
    assert!(
        !String::from_utf8_lossy(&no_main_output.stderr).contains("name not found: #"),
        "name error should not expose raw name ids\nstderr:\n{}",
        String::from_utf8_lossy(&no_main_output.stderr)
    );

    let missing_attr_arg = missing_attr_path.to_string_lossy().into_owned();
    let missing_attr_output = run_pabble(&["run", &missing_attr_arg], &dir);

    assert!(
        missing_attr_output.status.success(),
        "pabble should return after printing errors"
    );
    assert_stderr_contains(&missing_attr_output, "attribute not found: 'missing_attr'");
    assert!(
        !String::from_utf8_lossy(&missing_attr_output.stderr).contains("attribute not found: #"),
        "attribute error should not expose raw name ids\nstderr:\n{}",
        String::from_utf8_lossy(&missing_attr_output.stderr)
    );
}
