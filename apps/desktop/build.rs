fn main() {
    // macOS: add an rpath so the executable resolves the OpenCV dylibs that
    // `dylibbundler` copies into `<App>.app/Contents/Frameworks/` during the release
    // build (the teacher's Mac has no Homebrew OpenCV). dylibbundler rewrites the
    // dylib references to `@executable_path/../Frameworks/`, and this rpath covers any
    // `@rpath/`-relative names that slip through. Harmless on dev builds.
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");

    tauri_build::build()
}
