use xmp::*;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1) {
        println!("{:#?}", OptionalResults::load(path)?);
    }
    Ok(())
}
