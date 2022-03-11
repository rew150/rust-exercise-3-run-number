mod file;

fn main() {
    let h = file::open_file("./target/hello.txt", "a").unwrap();
    h.puts("Hello, world 2\n").unwrap();
}
