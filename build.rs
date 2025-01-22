use std::env;

fn main() {
    let mut build = cc::Build::new();
    let target = env::var("TARGET").unwrap();

    build
        .include("src/webview/ffi/lib.h")
        .file("src/webview/ffi/lib_impl.cpp")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-w")
        .cpp(true);

    if target.contains("apple") {
        build
            .file("src/webview/ffi/Platform_macOS.c")
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

        build.file("src/webview/ffi/Platform_Linux.cpp");
    } else {
        panic!("Unsupported target, make a pull request (or issue) if you want support for this operating system.");
    }

    build.compile("webview");
}
