mod cargo_arguments;
mod commands;
mod server_connection;
mod utils;

use std::process::ExitCode;

use remote_cargo_core::RemoteCargoError;

const USAGE: &str = "usage: cargo-remote <command>

  run [cargo args] [-- program args]   build on the server, run locally
  build [cargo args]                   build on the server, download executables
  clean [cargo args]                   clean the current project on the server and locally
  clear                                delete the build cache of all projects on the server
  space                                show disk space used by each project on the server
  prune [project-name]                 delete one project, or all projects, from the server
  setup                                install the build tools on the server";

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
        "clean" => commands::clean::execute(subcommand_arguments),
        "clear" => commands::clear::execute(),
        "space" => commands::space::execute(),
        "prune" => commands::prune::execute(subcommand_arguments),
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
