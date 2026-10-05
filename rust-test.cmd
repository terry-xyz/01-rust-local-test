@echo off
cargo run --quiet --offline --manifest-path "%~dp0Cargo.toml" --bin rust-test -- %*
exit /b %errorlevel%
