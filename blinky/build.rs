use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Add the linker files as flags to rustc
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    Ok(())
}
