use lib::{FontBackend, FontOptions, OpentypeTtfBackend, Workspace};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let workspace = Workspace::load("./examples/bitkodi")?;
    let doc = &workspace.projects[0].files[0].document;

    let builder = OpentypeTtfBackend::new(FontOptions {
        name: "bitkodi".to_string(),
        revision: 1.000,
        height: 8,
    });
    for glyph in doc.list_glyph() {
        println!();
        println!(
            "{}",
            glyph
                .labels
                .iter()
                .flat_map(|label| label.to_semantic().map(|s| s.to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let Some(value) = &glyph.value else {
            continue;
        };
        for row in &value.data {
            for col in row {
                if let Some(col) = col {
                    print!("{:x} ", col.value())
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!();
    }
    builder.build_to("./examples/bitkodi/dist")?;
    Ok(())
}
