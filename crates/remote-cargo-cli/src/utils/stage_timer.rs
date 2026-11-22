use std::time::Instant;

const VERBOSE_VARIABLE: &str = "REMOTE_CARGO_VERBOSE";

pub fn timed_stage<T>(stage_name: &str, stage: impl FnOnce() -> T) -> T {
    if std::env::var_os(VERBOSE_VARIABLE).is_none() {
        return stage();
    }
    let start = Instant::now();
    let result = stage();
    eprintln!("cargo-remote: {stage_name} took {:.2?}", start.elapsed());
    result
}
