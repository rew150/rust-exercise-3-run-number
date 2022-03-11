mod file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = file::open_file("./target/hello.txt", "r")?;
    loop {
        match h.gets(256) {
            Ok((s, eof)) => if eof {
                println!("last line: {}", s);
                break
            } else {
                print!("line: {}", s)
            },
            Err(e) => {
                println!("err, {}", e);
                break
            },
        }
    }
    println!("end");
    Ok(())
}
