use cargo_metadata::camino::Utf8Path;

pub fn common_ancestor<'a>(paths: impl IntoIterator<Item = &'a Utf8Path>) -> Option<&'a Utf8Path> {
    let mut paths = paths.into_iter();
    let first_path = paths.next()?;
    paths.try_fold(first_path, |ancestor, path| {
        ancestor
            .ancestors()
            .find(|candidate| path.starts_with(candidate))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_parent_of_sibling_directories() {
        let paths =
            ["/e/os/bar", "/e/os/ui/crates/view", "/e/os/vendor/encoding"].map(Utf8Path::new);
        assert_eq!(common_ancestor(paths), Some(Utf8Path::new("/e/os")));
    }

    #[test]
    fn keeps_single_directory_and_compares_whole_components() {
        assert_eq!(
            common_ancestor([Utf8Path::new("/e/api")]),
            Some(Utf8Path::new("/e/api"))
        );
        let paths = ["/e/api", "/e/api-client"].map(Utf8Path::new);
        assert_eq!(common_ancestor(paths), Some(Utf8Path::new("/e")));
    }
}
