mod cargo_arguments;
mod commands;
mod local_fallback;
mod utils;

use std::process::ExitCode;

use remote_cargo_core::RemoteCargoError;

const USAGE: &str =
    "usage: cargo-remote <run|build|setup> [cargo arguments] [-- program arguments]";

fn main() -> ExitCode {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|argument| argument == "remote")
    {
        arguments.remove(0);
    }
    let Some((subcommand, subcommand_arguments)) = arguments.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    let outcome = match subcommand.as_str() {
        "run" => commands::run::execute(subcommand_arguments),
        "build" => commands::build::execute(subcommand_arguments),
        "setup" => commands::setup::execute(),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cargo-remote: {error:#}");
            let exit_code = error
                .downcast_ref::<RemoteCargoError>()
                .and_then(RemoteCargoError::exit_code)
                .and_then(|code| u8::try_from(code).ok())
                .unwrap_or(1);
            ExitCode::from(exit_code)
        }
    }
}
