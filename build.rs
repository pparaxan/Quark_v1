use std::env;

fn main() {
    let mut build = cc::Build::new();
    let target = env::var("TARGET").unwrap();

    build.cpp(true);

    if target.contains("apple") {
        build.file("src/webview/webview.cc");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else if target.contains("linux") || target.contains("bsd") {
        let lib = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.1")
            .unwrap();

        for path in lib.include_paths {
            build.include(path);
        }

        build.file("src/webview/webview.cc");
    } else {
        panic!("Unsupported target, make a pull request (or issue) if you want support for this operating system.");
    }

    println!("cargo:rerun-if-changed=src/webview/webview.cc");
    build.compile("webview");
}
