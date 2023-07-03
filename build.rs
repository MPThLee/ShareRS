fn main() {
    // Rerun if migrations are changed
    println!("cargo:rerun-if-changed=migrations");

    // build-time information
    built::write_built_file().expect("Failed to acquire build-time information")
}
