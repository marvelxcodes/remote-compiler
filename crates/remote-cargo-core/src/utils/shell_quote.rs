use std::borrow::Cow;

pub fn shell_quote(argument: &str) -> Cow<'_, str> {
    let is_safe_unquoted = !argument.is_empty()
        && argument
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_-./=:,+@%".contains(character));
    if is_safe_unquoted {
        Cow::Borrowed(argument)
    } else {
        Cow::Owned(format!("'{}'", argument.replace('\'', r"'\''")))
    }
}

pub fn shell_join<'a>(arguments: impl IntoIterator<Item = &'a str>) -> String {
    arguments
        .into_iter()
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_safe_arguments_unquoted() {
        assert_eq!(shell_quote("--features=a,b"), "--features=a,b");
    }

    #[test]
    fn quotes_spaces_and_single_quotes() {
        assert_eq!(shell_quote("it's here"), r"'it'\''s here'");
        assert_eq!(shell_quote(""), "''");
    }
}
