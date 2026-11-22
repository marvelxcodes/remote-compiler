# Remote Compiler for Rust

`cargo-remote` compiles your Rust projects on a remote Linux server and runs the binaries on your local machine.

Type `cargo run` or `cargo build` as usual. The tool sends the source to the server, compiles it there, and copies only the executables back. The full build cache (`target/`) stays on the server.


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

`REMOTE_CARGO_HOST`: SSH destination of the build server, for example `user@100.64.0.10`. 
(If it is not set, `run` and `build` use the local `cargo`)

`REMOTE_CARGO_VERBOSE`: Set to any value to show the duration of each stage (connect, upload, remote build, download).
