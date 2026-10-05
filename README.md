# 01-rust-local-test

Run the official 01-edu Rust exercise tests locally with Cargo, Rust's build
and test tool. No Docker or Node.js is needed for normal runs.

The bundled files come unchanged from the official
[01-edu Rust test image](https://github.com/01-edu/rust-tests).
Use `--list` to see the 146 bundled exercise suites.

## Requirements

Install Rust with [rustup](https://rustup.rs/), then open a new terminal.
Check that both commands work:

```powershell
rustc --version
cargo --version
```

Some exercises use external crates, which Cargo downloads on their first run.
Other exercises may need nightly Rust or a Unix system, as the official image
uses Linux and nightly Rust. This local runner does not reproduce the image's
process isolation or memory and CPU limits.

**Run only trusted code.** Use this tester for solutions you wrote yourself or
code you have reviewed and trust. It runs solutions, tests, and build scripts
with your computer account's permissions. They can access your files, network,
and environment variables. A temporary copy does not prevent that access.

## Usage

Use this tool to check your exercise before submitting it to 01-edu.
Write and save your solution in `piscine-rust`, then run the matching exercise
test. Replace `scalar` in the commands below with the exercise you want to check.

The examples assume the tester and your piscine folder are next to each other:

```text
Desktop/
  01-rust-local-test/
  piscine-rust/
    scalar/
      src/
        lib.rs
```

If `01-rust-local-test` is in another directory, replace `..` in the commands
with the matching path from your current folder.

### PowerShell: already inside piscine-rust

If your terminal is open in `piscine-rust`, run:

```powershell
..\01-rust-local-test\rust-test.cmd scalar
```

You do not need to give a solution path. The tester looks for
`scalar\src\lib.rs` inside your current folder.

### PowerShell: inside 01-rust-local-test

If your terminal is open in the tester folder, run:

```powershell
.\rust-test.cmd scalar ..\piscine-rust
```

Here, `scalar` is the exercise name and `..\piscine-rust` tells the tester where
your solutions are saved.

You can also give the exercise folder or its source file:

```powershell
.\rust-test.cmd scalar ..\piscine-rust\scalar
.\rust-test.cmd scalar ..\piscine-rust\scalar\src\lib.rs
```

### PowerShell: inside piscine-rust\scalar

If you have opened the terminal inside the exercise folder itself, run:

```powershell
..\..\01-rust-local-test\rust-test.cmd scalar
```

### Git Bash, macOS, or Linux

Use `/` in paths and the `rust-test` launcher. From inside `piscine-rust`, run:

```sh
sh ../01-rust-local-test/rust-test scalar
```

From inside `01-rust-local-test`, run:

```sh
sh ./rust-test scalar ../piscine-rust
```

### Read the result

```text
test result: ok. 4 passed; 0 failed; ...
scalar passed
```

This means your `scalar` solution passed all four official tests in the bundled
version. Other exercises may have a different number of tests.

If you see `FAILED` or `scalar failed`, read the failed test's message, fix your
solution, save it, and run the same command again. If you see a compiler error,
Rust could not build your code, so fix that error first.

Some tests include `should panic`. These tests expect an error, such as integer
overflow. A line ending in `should panic ... ok` means that test passed.

### See which exercises are available

From inside `piscine-rust`, run:

```powershell
..\01-rust-local-test\rust-test.cmd --list
```

From inside the tester folder, run:

```powershell
.\rust-test.cmd --list
```

For Git Bash, macOS, or Linux, add `--list` after the launcher instead of an
exercise name. Exercise names accept hyphens or underscores.

### Common problems

- **`cargo` is not recognized:** install Rust, then close and reopen your terminal.
- **The launcher cannot be found:** check which folder your terminal is in.
  The relative paths above assume the two project folders are next to each other.
  If they are elsewhere, use their full paths in quotes:

  ```powershell
  & "C:\your folder\01-rust-local-test\rust-test.cmd" scalar "C:\your folder\piscine-rust"
  ```

- **No upstream tests bundled:** run `--list` and check the exercise name.
- **No solution folder found:** check that your exercise has a `src` folder and
  that you gave the correct solution path.
- **Timed out:** runs stop after two minutes, including downloads, compilation,
  and tests. Check your code for an endless loop. If a trusted build needs more
  time, set a longer limit before running it:

  ```powershell
  $env:RUST_TEST_TIMEOUT_SECS = '300'
  ..\01-rust-local-test\rust-test.cmd scalar
  Remove-Item Env:\RUST_TEST_TIMEOUT_SECS
  ```

  In Git Bash, macOS, or Linux:

  ```sh
  RUST_TEST_TIMEOUT_SECS=300 sh ../01-rust-local-test/rust-test scalar
  ```

  The value must be a positive whole number of seconds. When the limit is
  reached, the runner stops Cargo and its child processes.

The tester copies your solution before running it. If your exercise has no
`Cargo.toml`, the file Rust uses for project settings, the tester creates one
only in a temporary copy. Some tests also need another solved exercise next to
the current one, such as `expected_variable` needing `edit_distance`.

Solutions and tests are copied into a temporary directory and cleaned up after
each run, including timeouts. On Linux and macOS, only your account can access
this directory. Build output is kept in this repository's ignored `target`
directory.
The command returns a nonzero exit code when compilation or tests fail.

## Upstream files

[source.json](vendor/01-edu-rust/source.json) identifies the pinned official
image. [THIRD_PARTY.md](THIRD_PARTY.md) records provenance.
No exercise tests are authored by this project.

To restore the pinned test files, use Python 3.9 or newer:

```powershell
python scripts/sync-upstream.py
```

The script verifies image and layer hashes before extracting tests and their
shared helper. To update the snapshot, change the image digest in `source.json`,
run the script, review the vendor changes, and rerun exercises you have solved.

## Check the runner

```powershell
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
.\rust-test.cmd scalar ..\piscine-rust
```

The CLI is defined in [src/main.rs](src/main.rs); exercise assertions remain
in the [official test suites](vendor/01-edu-rust/tests).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, checks, and how to submit changes.

## License

Original project files are covered by the [MIT license](LICENSE). Vendored
01-edu files retain their upstream terms and notices; see [THIRD_PARTY.md](THIRD_PARTY.md).
