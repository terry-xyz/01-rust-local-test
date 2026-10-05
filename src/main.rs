use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, ExitStatus};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const USAGE: &str = "usage: rust-test <exercise> [solution-directory-or-file]\n       rust-test --list\n\nexamples:\n  rust-test scalar ../piscine-rust\n  rust-test scalar ../piscine-rust/scalar/src/lib.rs\n\nRun only trusted code. This tool does not sandbox solutions or build scripts.\nRuns stop after 120 seconds; set RUST_TEST_TIMEOUT_SECS to a positive whole number to change the limit.";

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> io::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("01-rust-local-test-{}-{timestamp}", process::id()));
        let builder = fs::DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        builder.create(&path)?;
        Ok(Self(path))
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!(
                "Could not remove temporary directory {}: {error}",
                self.0.display()
            );
        }
    }
}

fn stop_process_tree(child: &mut Child) -> io::Result<()> {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        // The child starts its own process group, so this also stops its descendants.
        if unsafe { kill(-(child.id() as i32), 9) } != 0 {
            let error = io::Error::last_os_error();
            if child.try_wait()?.is_none() {
                return Err(error);
            }
        }
    }
    #[cfg(windows)]
    {
        let system_root = env::var_os("SystemRoot")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "SystemRoot is not set"))?;
        let status = Command::new(PathBuf::from(system_root).join("System32/taskkill.exe"))
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()?;
        if !status.success() && child.try_wait()?.is_none() {
            return Err(io::Error::other(
                "Could not stop Cargo and its child processes",
            ));
        }
    }
    #[cfg(not(any(unix, windows)))]
    child.kill()?;
    child.wait()?;
    Ok(())
}

fn run_with_timeout(command: &mut Command, timeout: Duration) -> io::Result<ExitStatus> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => {
                stop_process_tree(&mut child)?;
                return Err(error);
            }
        }
        if started.elapsed() >= timeout {
            stop_process_tree(&mut child)?;
            return Err(io::Error::new(io::ErrorKind::TimedOut, format!(
                "Timed out after {} seconds. Cargo and its child processes were stopped. Set RUST_TEST_TIMEOUT_SECS to allow more time for a slow build.", timeout.as_secs()
            )));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Symlinks are not supported: {}", entry.path().display()),
            ));
        }
        if kind.is_dir() {
            copy_tree(&entry.path(), &destination.join(name))?;
        } else {
            fs::copy(entry.path(), destination.join(name))?;
        }
    }
    Ok(())
}

fn solution_dir(exercise: &str, input: &Path) -> io::Result<PathBuf> {
    let input = input.canonicalize()?;
    let directory = if input.is_file() {
        if !input.parent().is_some_and(|parent| parent.ends_with("src")) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Solution file must be inside an exercise's src directory",
            ));
        }
        input.parent().unwrap().parent().unwrap().to_path_buf()
    } else if input.join(exercise).is_dir() {
        input.join(exercise)
    } else {
        input
    };
    if !directory.join("src").is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "No {exercise}/src or src directory found in {}",
                directory.display()
            ),
        ));
    }
    Ok(directory)
}

fn stage_solution(source: &Path, destination: &Path, name: &str) -> io::Result<()> {
    copy_tree(source, destination)?;
    if !destination.join("Cargo.toml").is_file() {
        fs::write(
            destination.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )?;
    }
    Ok(())
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("{USAGE}");
        return Ok(0);
    }
    let vendor = Path::new(ROOT).join("vendor/01-edu-rust");
    let tests_dir = vendor.join("tests");
    if args.len() == 1 && args[0] == "--list" {
        let mut names = Vec::new();
        for entry in fs::read_dir(&tests_dir)? {
            let entry = entry?;
            if entry.path().join("Cargo.toml").is_file() {
                if let Some(name) = entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.strip_suffix("_test"))
                {
                    names.push(name.to_owned());
                }
            }
        }
        names.sort();
        println!("{}", names.join("\n"));
        return Ok(0);
    }
    if args.is_empty() || args.len() > 2 {
        return Err(USAGE.into());
    }
    let name = args[0]
        .to_str()
        .ok_or("Exercise name must be valid UTF-8")?;
    let exercise = name.strip_suffix(".rs").unwrap_or(name).replace('-', "_");
    if exercise.is_empty()
        || !exercise
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || exercise.as_bytes()[0].is_ascii_digit()
    {
        return Err("Invalid exercise name: use letters, digits, underscores, or hyphens; start with a letter or underscore".into());
    }
    let tests = tests_dir.join(format!("{exercise}_test"));
    if !tests.join("Cargo.toml").is_file() {
        return Err(
            format!("No upstream tests bundled for '{exercise}'. Run rust-test --list.").into(),
        );
    }
    let input = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let source = solution_dir(&exercise, &input)?;
    let timeout_seconds = match env::var("RUST_TEST_TIMEOUT_SECS") {
        Ok(value) => value
            .parse::<u64>()
            .ok()
            .filter(|seconds| *seconds > 0)
            .ok_or("RUST_TEST_TIMEOUT_SECS must be a positive whole number")?,
        Err(env::VarError::NotPresent) => 120,
        Err(error) => return Err(error.into()),
    };
    let workspace = Workspace::new()?;
    let project = workspace.0.join("tests").join(format!("{exercise}_test"));
    copy_tree(&tests, &project)?;
    copy_tree(&vendor.join("tests/lib"), &workspace.0.join("tests/lib"))?;
    let solutions = workspace.0.join("solutions");
    stage_solution(&source, &solutions.join(&exercise), &exercise)?;

    let manifest_path = project.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    // The upstream manifests use this fixed relative path for student crates.
    for suffix in manifest.split("../../solutions/").skip(1) {
        let name = suffix.split('"').next().unwrap().trim_end_matches('/');
        if name != exercise {
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err("Unexpected student dependency path in upstream manifest".into());
            }
            let sibling = source.parent().unwrap().join(name);
            stage_solution(&sibling, &solutions.join(name), name)?;
        }
    }
    fs::write(
        &manifest_path,
        format!("{manifest}\n[workspace]\n[profile.test]\noverflow-checks = true\n"),
    )?;
    println!("Testing {exercise} with 01-edu tests: {}", source.display());
    let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command
        .args(["test", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(Path::new(ROOT).join("target/exercises"))
        .env("CARGO_TARGET_DIR", Path::new(ROOT).join("target/exercises"))
        .current_dir(&project);
    let status = run_with_timeout(&mut command, Duration::from_secs(timeout_seconds))?;
    if status.success() {
        println!("{exercise} passed");
    } else {
        eprintln!("{exercise} failed");
    }
    Ok(status.code().unwrap_or(1))
}

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_command(mode: &str) -> Command {
        let mut command = Command::new(env::current_exe().unwrap());
        command
            .args(["--exact", "tests::process_fixture", "--nocapture"])
            .env("RUST_TEST_PROCESS_FIXTURE", mode)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());
        command
    }

    #[test]
    fn process_fixture() {
        match env::var("RUST_TEST_PROCESS_FIXTURE").as_deref() {
            Ok("fail") => panic!("Fixture failure"),
            Ok("parent") => {
                fixture_command("child").spawn().unwrap().wait().unwrap();
            }
            Ok("child") => {
                for _ in 0..500 {
                    fs::write(
                        env::var_os("RUST_TEST_HEARTBEAT").unwrap(),
                        format!("{:?}", SystemTime::now()),
                    )
                    .unwrap();
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
            _ => {}
        }
    }

    #[test]
    fn process_exit_codes_and_timeout() {
        assert!(
            run_with_timeout(&mut fixture_command("pass"), Duration::from_secs(5))
                .unwrap()
                .success()
        );
        assert!(
            !run_with_timeout(&mut fixture_command("fail"), Duration::from_secs(5))
                .unwrap()
                .success()
        );
        let workspace = Workspace::new().unwrap();
        let heartbeat = workspace.0.join("heartbeat");
        let error = run_with_timeout(
            fixture_command("parent").env("RUST_TEST_HEARTBEAT", &heartbeat),
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        let last = fs::read(&heartbeat).expect("Child process did not start");
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(
            fs::read(&heartbeat).unwrap(),
            last,
            "Descendant survived timeout"
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let workspace = Workspace::new().unwrap();
        assert_eq!(
            fs::metadata(&workspace.0).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
