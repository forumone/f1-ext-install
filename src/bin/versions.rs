// Helper program to output the package versions as determined by Cargo.
// (This helps avoid some duplication in the build scripts with tagging)

fn main() {
    let major = env!("CARGO_PKG_VERSION_MAJOR");
    let minor = env!("CARGO_PKG_VERSION_MINOR");
    let patch = env!("CARGO_PKG_VERSION_PATCH");

    println!("{}.{}.{}", major, minor, patch);
    println!("{}.{}", major, minor);
    println!("{}", major);
}
