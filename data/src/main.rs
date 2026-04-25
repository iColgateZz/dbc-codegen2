use data::*;

fn main() -> Result<(), CanError> {
    let msg = ExampleMessage::new(231.0, 3.0, true)?;

    println!("{}", msg.temperature());

    Ok(())
}