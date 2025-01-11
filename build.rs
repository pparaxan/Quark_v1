use std::env;

fn main() {
    let mut build = cc::Build::new();
    let target = env::var("TARGET").unwrap();

    build
        .include("src/webview/ffi/webview.h")
        .file("src/webview/ffi/webview_impl.cpp")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-w")
        .cpp(true);

    if target.contains("apple") {
        build.file("src/webview/ffi/platform_darwin.c")
            .flag("-x")
            .flag("objective-c");

        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else if target.contains("linux") || target.contains("bsd") {
        let lib = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.1")
            .expect("Quark can't find webkit2gtk-4.1! Please install it.");

        for path in &lib.include_paths {
            build.include(path);
        }

        build.file("src/webview/ffi/platform_linux.cpp");
    } else if target.contains("windows") {
        build.define("UNICODE", None);

        build
            .file("src/webview/ffi/platform_nt.cpp")
            .flag("-m64")
            .flag_if_supported("/std:c++17")
            .flag_if_supported("/EHsc");

        for &lib in &["advapi32", "ole32", "shell32", "shlwapi", "user32", "version"] {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else {
        panic!("Unsupported target, make a pull request (or issue) if you want support for this operating system.");
    }

    build.compile("webview");
}
