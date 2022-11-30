fn main() {
    #[cfg(all(windows, target_env = "msvc"))]
    static_vcruntime::metabuild()
}
