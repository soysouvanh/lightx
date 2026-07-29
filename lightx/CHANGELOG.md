# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-29

### Added

- Phase 1 Architecture (CLI, Generator, Sandbox Testing)
- Initial Zero-Mock framework foundation.
- Strict CI/CD SecOps integration.

### Security

- Enforced `AppError::SystemError` over `unwrap()` panic on missing `JWT_SECRET` environments.
- Sanitized `Cargo.toml` to forcefully exclude fuzz testing binaries from published crates.

### Fixed

- Surgically isolated `SuperTest` mock framework and `MockRouter` code generation behind the `testing` Cargo feature flag.
- Purged raw `println!` traces from network listener in favor of O(1) unified telemetry logger.
- Removed arbitrary and blocking synchronous I/O test (`.lightx_write_test`) during logger initialization.
- Renamed development stubs (`MockRouter`/`DummyRouter`) to professional equivalents in documentation tests.
