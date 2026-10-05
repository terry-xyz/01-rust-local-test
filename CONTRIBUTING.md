# Contributing to 01-rust-local-test

Contributions are welcome. Keep the tester simple, easy for students to use,
and based on the official 01-edu exercise tests.

## Getting Started

1. Fork [the repository](https://github.com/terry-xyz/01-rust-local-test).
2. Clone your fork:

   ```sh
   git clone https://github.com/YOUR_USERNAME/01-rust-local-test.git
   cd 01-rust-local-test
   ```

3. Create a branch for your change:

   ```sh
   git switch -c codex/your-change
   ```

## Development Requirements

- Install Rust and Cargo using [rustup](https://rustup.rs/).
- Have the `rustfmt` formatter and Clippy checker installed.
- Python 3.9 or newer is needed only to refresh the upstream test files.

The runner uses Rust's standard library and requires no extra Rust packages.
See [README.md](README.md) for student usage and platform requirements.

## Code Standards

- Keep changes small and readable; reuse existing code where possible.
- Run `cargo fmt` before committing.
- Add a small runner regression check in `src/main.rs` when changing behavior.
- Keep the official exercise tests unchanged. Do not replace them with your
  own exercise tests or reformat vendored files.
- Preserve private temporary folders, process cleanup, and timeout behavior.
- Update the README when commands or requirements change.

## Testing

From the project root, run:

```sh
cargo fmt --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
```

Check an exercise against a solution you wrote or reviewed and trust. If
`piscine-rust` is next to this repository, use PowerShell:

```powershell
.\rust-test.cmd scalar ..\piscine-rust
```

Or Git Bash, macOS, or Linux:

```sh
sh ./rust-test scalar ../piscine-rust
```

This command uses the official tests. Do not commit student solutions into
the tester repository. When changing process handling, check both Windows and
Linux where available.

## Building

```sh
cargo build --offline
```

## Submitting Changes

1. Run the checks above and fix any failures.
2. Use a clear commit message, such as `fix: stop child processes on timeout`
   or `docs: explain how to run an exercise`.
3. Push your branch to your fork.
4. Open a pull request against `main`. Explain what changed, why, and which
   checks you ran.

## Project Structure

- [src/main.rs](src/main.rs): command handling, temporary copies, timeouts,
  and runner checks.
- `rust-test` and `rust-test.cmd`: shell and Windows launchers.
- [Cargo.toml](Cargo.toml): Rust project settings.
- [scripts/sync-upstream.py](scripts/sync-upstream.py): restores official tests
  from the pinned image.
- [vendor/01-edu-rust](vendor/01-edu-rust): official exercise tests and helpers.
- [THIRD_PARTY.md](THIRD_PARTY.md): upstream source and license notices.

## Code Review

Pull requests should pass the checks, keep upstream tests recognizable, and
include relevant documentation updates. Keep security protections in place.

Contributions to the original project files are covered by the [MIT license](LICENSE).
Upstream files retain their own terms and notices.

## Questions?

Open an issue in the [issue tracker](https://github.com/terry-xyz/01-rust-local-test/issues).
