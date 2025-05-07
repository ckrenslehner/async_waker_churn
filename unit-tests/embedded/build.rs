use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Add the linker files as flags to rustc

    // PREVIOUS
    // println!("cargo:rustc-link-arg-bins=--nmagic");
    // println!("cargo:rustc-link-arg-bins=-Tlink.x");
    // println!("cargo:rerun-if-changed=link.x");
    // println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // NOW
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // Add linker file of embedded-test crate only for tests
    println!("cargo::rustc-link-arg-tests=-Tembedded-test.x");
    
    Ok(())
}
