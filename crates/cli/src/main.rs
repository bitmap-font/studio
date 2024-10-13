use lib::Workspace;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let workspace = Workspace::load("./examples/bitkodi")?;
    println!("Hello, world!");
    Ok(())
}
