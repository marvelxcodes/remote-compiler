use cargo_metadata::camino::Utf8Path;

pub fn remote_directory_name(local_workspace_root: &Utf8Path) -> String {
    local_workspace_root
        .as_str()
        .trim_start_matches('/')
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' => character,
            _ => '_',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_absolute_path_into_single_directory_name() {
        assert_eq!(
            remote_directory_name(Utf8Path::new("/e/Projects/my app")),
            "e_Projects_my_app"
        );
    }
}
