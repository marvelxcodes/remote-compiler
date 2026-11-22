# remote-compiler

`cargo-remote` compiles your Rust projects on a remote Linux server and runs the binaries on your local machine.

Type `cargo run` or `cargo build` as usual. The tool sends the source to the server, compiles it there, and copies only the executables back. The full build cache (`target/`) stays on the server.

## How it works

```
local machine                                   build server
─────────────                                   ────────────
cargo run --release -- args
  │
  ├─ 1. rsync source  ───────────────────────▶  ~/.cache/remote-cargo/<project>/
  │     (no target/, .git/, gitignored files)
  │
  ├─ 2. ssh: cargo build --release  ─────────▶  compiles, keeps target/ cache
  │     (diagnostics stream back to you)
  │
  ├─ 3. rsync executable  ◀──────────────────   target/release/<bin>
  │     into local target/release/<bin>
  │
  └─ 4. exec <bin> args   (runs locally)
```

- The server reads your synced `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` and `.cargo/config.toml`. When you change them locally, the next run recompiles with the new settings.
- All `ssh` and `rsync` calls share one multiplexed connection. A run with no changes takes less than one second of overhead.
- If the server is not available, the tool shows a warning and runs the local `cargo` instead.
- Only `run` and `build` go to the server. All other commands (`check`, `test`, `clippy`) run locally.

## Tech stack

| Part | Technology |
| --- | --- |
| Language | Rust (edition 2024), Cargo workspace |
| Library crate | `remote-cargo-core`: [`cargo_metadata`](https://crates.io/crates/cargo_metadata) for workspace metadata and JSON build messages, [`thiserror`](https://crates.io/crates/thiserror) for errors |
| CLI crate | `remote-cargo-cli` (binary `cargo-remote`): [`anyhow`](https://crates.io/crates/anyhow) for errors |
| Transport | OpenSSH (with `ControlMaster` connection sharing) and `rsync` (delta transfer, compression) |
| Network | Any SSH route. Tested with [Tailscale](https://tailscale.com). |
| Shell integration | fish function that wraps `cargo` |

## Requirements

Local machine:
- Linux on x86_64
- Rust toolchain, `ssh`, `rsync` 3.2 or later
- fish shell (only for the `cargo` wrapper)

Build server:
- Ubuntu or Debian on x86_64
- SSH access with a key (no password prompt)

The server must use the same CPU architecture as your local machine. Do not set `target-cpu=native` in the project, because the server CPU can be different from the local CPU.

## Installation

1. Install the binary:

   ```sh
   cargo install --git https://github.com/marvelxcodes/remote-compiler remote-cargo-cli
   ```

2. Set up key authentication to the server (one time):

   ```sh
   ssh-keygen -t ed25519
   ssh-copy-id user@build-server
   ```

3. Set the server address:

   ```fish
   set -gx REMOTE_CARGO_HOST user@build-server
   ```

4. Prepare the server. This command installs `build-essential`, `pkg-config`, `libssl-dev`, `rsync`, `curl` and `rustup`. Run it in a real terminal, because `sudo` asks for a password:

   ```sh
   cargo-remote setup
   ```

   You can run `setup` again at any time. It also updates the stable toolchain.

5. Install the fish wrapper:

   ```sh
   cp shell/remote-cargo.fish ~/.config/fish/conf.d/
   ```

   Edit `REMOTE_CARGO_HOST` in that file, then open a new shell.

## Usage

With the fish wrapper:

```sh
cargo run                         # compile on the server, run locally
cargo run --release -- --port 80  # arguments after -- go to your program
cargo run -p server --bin api     # select a package and a binary
cargo build --release             # copy all built executables to target/release/
```

Without the wrapper, use the Cargo subcommand form:

```sh
cargo remote run --release
cargo-remote build
```

To use local `cargo` for one command, type `command cargo run`.

## Configuration

| Variable | Description |
| --- | --- |
| `REMOTE_CARGO_HOST` | SSH destination of the build server, for example `user@100.64.0.10`. If it is not set, `run` and `build` use the local `cargo`. |
| `REMOTE_CARGO_VERBOSE` | Set to any value to show the duration of each stage (connect, upload, remote build, download). |

## Known limitations

- The tool does not upload gitignored files (for example `.env` or generated code) or `.git/`.
- Path dependencies outside the workspace root are not uploaded. The tool stops with an error.
- Compile-time paths (`file!()`, `env!("CARGO_MANIFEST_DIR")`, panic locations) show server paths. At runtime, `CARGO_MANIFEST_DIR` and `CARGO_PKG_*` contain the local values.
- Dynamically linked system libraries (for example `libssl`) must also be installed locally. Use `ldd target/release/<bin>` to make sure.
- `--manifest-path` with a local absolute path does not work.

## Project layout

```
crates/
  remote-cargo-core/      library: ssh session, source sync, remote build, artifact download
    src/utils/            shell quoting, remote directory names
  remote-cargo-cli/       binary cargo-remote: argument parsing, run/build/setup commands, local fallback
shell/remote-cargo.fish   fish wrapper for cargo
```

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
