use cargo_metadata::camino::Utf8Path;

pub fn containing_workspace_root(package_directory: &Utf8Path) -> &Utf8Path {
    package_directory
        .ancestors()
        .find(|directory| declares_workspace(&directory.join("Cargo.toml")))
        .unwrap_or(package_directory)
}

fn declares_workspace(manifest_path: &Utf8Path) -> bool {
    std::fs::read_to_string(manifest_path).is_ok_and(|manifest| {
        manifest
            .lines()
            .any(|line| line.trim_start().starts_with("[workspace"))
    })
}
