use std::process::Command;

fn main() {
    for pkg in ["gtk4", "libcurl", "gdk-pixbuf-2.0"] {
        let out = Command::new("pkg-config")
            .args(["--libs", pkg])
            .output()
            .unwrap_or_else(|e| panic!("failed to run pkg-config for {pkg}: {e}"));
        if !out.status.success() {
            panic!("pkg-config could not find {pkg}. Install its development package.");
        }
        let libs = String::from_utf8(out.stdout).expect("pkg-config output was not UTF-8");
        for arg in libs.split_whitespace() {
            println!("cargo:rustc-link-arg={arg}");
        }
    }
}
