use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");
const USAGE: &str = "usage: rust-test <exercise> [solution-directory-or-file]\n       rust-test --list\n\nexamples:\n  rust-test scalar ../piscine-rust\n  rust-test scalar ../piscine-rust/scalar/src/lib.rs";

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> io::Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("01-rust-local-test-{}-{timestamp}", process::id()));
        fs::create_dir(&path)?;
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
    let status = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(["test", "--manifest-path"])
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(Path::new(ROOT).join("target/exercises"))
        .env("CARGO_TARGET_DIR", Path::new(ROOT).join("target/exercises"))
        .current_dir(&project)
        .status()?;
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
