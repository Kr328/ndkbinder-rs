use std::path::PathBuf;

fn main() {
    let cc = std::env::var("TARGET_CC")
        .or_else(|_| std::env::var(format!("CC_{}", std::env::var("TARGET").unwrap())))
        .expect(&format!("CC not found for target {}", std::env::var("TARGET").unwrap()));
    let sysroot = PathBuf::from(cc)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("sysroot"))
        .filter(|p| p.exists());

    let mut args = Vec::new();
    if let Some(ref sysroot) = sysroot {
        args.push(format!("--sysroot={}", sysroot.to_str().unwrap()));
    }

    bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("unable to generate binder bindings")
        .write_to_file(std::env::var("OUT_DIR").unwrap() + std::path::MAIN_SEPARATOR_STR + "binder_sys.rs")
        .unwrap();

    println!("cargo:rustc-link-arg=-lbinder_ndk");
    println!("cargo:rerun-if-changed=wrapper.h");
}
