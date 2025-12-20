//! Build script for Windows resource embedding
//!
//! Adds Windows manifest for DPI awareness and application metadata.

fn main() {
    // Only run for Windows builds
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // Check if winres is available
        #[cfg(feature = "winres")]
        {
            let mut res = winres::WindowsResource::new();
            res.set_manifest_file("gamevault.manifest");
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}", e);
            }
        }

        // Fallback: just set the manifest
        println!("cargo:rerun-if-changed=gamevault.manifest");
    }

    // Rerun if frontend build changes
    println!("cargo:rerun-if-changed=../frontend/out");
}
