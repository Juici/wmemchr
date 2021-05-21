fn main() {
    autocfg::rerun_path("build.rs");

    let cfg = autocfg::new();

    // Check stability of `NonZeroI32::trailing_zeros` and
    // `NonZeroI32::leading_zeros` (rustc 1.53+).
    cfg.emit_rustc_version(1, 53);
}
