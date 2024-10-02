fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/data/icon.ico");
        res.compile().expect("Failed to compile resources");
    }
    println!("cargo:rustc-env=VERSION=1.0");
}
