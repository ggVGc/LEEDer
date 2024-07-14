fn main() {
    cc::Build::new()
        .file("src/camera/native/api.cpp")
        .compiler("g++")
        .compile("netusbcam_api");

    println!("cargo:rustc-link-lib=NETUSBCAM");
}
