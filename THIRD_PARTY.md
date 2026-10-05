# Third-party code

The [MIT license](LICENSE) covers this project's original runner, scripts,
tests, documentation, and metadata. Vendored files retain their upstream
terms and notices.

Files under `vendor/01-edu-rust/tests` and `vendor/01-edu-rust/tests_utility`
are copied unchanged from the official `ghcr.io/01-edu/test-rust` image.

- Source: [01-edu/rust-tests](https://github.com/01-edu/rust-tests).
- Image digest: `sha256:979f5a497a90eb55ce610afcf87cdd01fea1ca8694a9cc98e6db22c028ea3e82`.
- Test layer: `sha256:6f1d13ef83e2b8e21e9f0d0e9e2225e88a71a61916af45cfc24410c5fc4c587c`.
- Helper layer: `sha256:22e30301760f2acfc7c559755ed79532969b8ee607643a0a25dd1c53d78f4ba6`.
- The official image declares `org.opencontainers.image.licenses=MIT`.

Only test suites and their helper are included. The runner adjusts the layout
and Cargo settings in a temporary copy; vendored files remain unchanged.
[scripts/sync-upstream.py](scripts/sync-upstream.py) reproduces the extraction
from the pinned image without running Docker.
