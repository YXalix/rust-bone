

fn setup_linking() {
    let obmm_lib_path = std::fs::canonicalize("obmm-sys/build").unwrap();
    println!("cargo:rustc-link-search=native={}", obmm_lib_path.display());
    if std::env::consts::OS == "linux" {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            obmm_lib_path.display()
        );
    } else if std::env::consts::OS == "windows" {
        println!(
            "cargo:rustc-link-arg=/LIBPATH:{}",
            obmm_lib_path.display()
        );
    } else if std::env::consts::OS == "macos" {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            obmm_lib_path.display()
        );
    }
    println!("cargo:rustc-link-lib=obmm");
}

fn main() {
    println!("Hello from obmm-rs build script!");
    setup_linking();
}