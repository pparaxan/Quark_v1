use std::env;

fn main() {
    let mut build = cc::Build::new();

    let target = env::var("TARGET").unwrap();

    if target.contains("apple") {
        build.file("webview.cc").cpp(true);
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else if target.contains("linux") || target.contains("bsd") {
        let lib = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0") // update this to latest, it's like at version 4.1 now
            .unwrap();

        for path in lib.include_paths {
            build.include(path);
        }

        build.file("webview.cc").cpp(true);
    } else {
        panic!("Unsupported target, make a issue if you want support for this operating system."); // I wonder if OSes like React, Haiku and Redox has webview...
    }

    println!("cargo:rerun-if-changed=ext/webview.h");
    println!("cargo:rerun-if-changed=ext/webview.cc");

    build.compile("webview");
}
