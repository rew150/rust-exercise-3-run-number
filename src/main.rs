mod file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut h = file::open_file("./target/hello.txt", "r")?;
    let pos = h.current_pos()?;
    let res = h.read_until_char(pos, 110)?;
    println!("{}\neof: {}", res.0, res.1);
    let pos2 = h.current_pos()?;
    let res = h.read_until_char(pos2, 0)?;
    println!("{}\neof: {}", res.0, res.1);
    println!("end");
    Ok(())
}
