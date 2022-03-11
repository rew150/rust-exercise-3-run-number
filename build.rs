fn main() {
    cc::Build::new()
        .file("cpp/main.cpp")
        .compile("run_number")
}
