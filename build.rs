fn main() {
    cc::Build::new()
        .cpp(true)
        .file("cpp/main.cpp")
        .compile("run_number")
}
