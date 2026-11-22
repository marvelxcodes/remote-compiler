use remote_cargo_core::executable_selection::ExecutableRequest;

pub fn split_program_arguments(arguments: &[String]) -> (&[String], &[String]) {
    match arguments.iter().position(|argument| argument == "--") {
        Some(separator_index) => (
            &arguments[..separator_index],
            &arguments[separator_index + 1..],
        ),
        None => (arguments, &[]),
    }
}

pub fn executable_request(cargo_arguments: &[String]) -> ExecutableRequest<'_> {
    let flag_value = |flag: &str| {
        cargo_arguments
            .iter()
            .enumerate()
            .find_map(|(index, argument)| {
                argument
                    .strip_prefix(flag)
                    .and_then(|rest| rest.strip_prefix('='))
                    .or_else(|| {
                        (argument == flag)
                            .then(|| cargo_arguments.get(index + 1).map(String::as_str))
                            .flatten()
                    })
            })
    };
    if let Some(binary_name) = flag_value("--bin") {
        ExecutableRequest::Binary(binary_name)
    } else if let Some(example_name) = flag_value("--example") {
        ExecutableRequest::Example(example_name)
    } else {
        ExecutableRequest::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(arguments: &[&str]) -> Vec<String> {
        arguments
            .iter()
            .map(|argument| argument.to_string())
            .collect()
    }

    #[test]
    fn splits_at_first_double_dash() {
        let arguments = owned(&["--release", "--", "a", "--", "b"]);
        let (cargo_arguments, program_arguments) = split_program_arguments(&arguments);
        assert_eq!(cargo_arguments, ["--release"]);
        assert_eq!(program_arguments, ["a", "--", "b"]);
    }

    #[test]
    fn finds_binary_in_both_flag_styles() {
        let separate = owned(&["--bin", "server"]);
        let joined = owned(&["--bin=server"]);
        assert!(matches!(
            executable_request(&separate),
            ExecutableRequest::Binary("server")
        ));
        assert!(matches!(
            executable_request(&joined),
            ExecutableRequest::Binary("server")
        ));
    }

    #[test]
    fn ignores_flags_with_similar_prefix() {
        let arguments = owned(&["--bins"]);
        assert!(matches!(
            executable_request(&arguments),
            ExecutableRequest::Default
        ));
    }
}
