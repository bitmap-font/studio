use lib::{FontBackend, FontOptions, FontVerseion, OpentypeTtfBackend, Workspace};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let workspace = Workspace::load("./examples/bitkodi")?;
    let doc = &workspace.projects[0].files[0].document;

    let mut builder = OpentypeTtfBackend::new(FontOptions {
        copyright_notice: None,
        family_name: "Bitkodi".to_string(),
        sub_family_name: "Regular".to_string(),
        version: FontVerseion::new(1, 0).unwrap(),
        unique_id: "bitkodi-test".to_owned(),
        full_font_name: None,
        postscript_name: None,
        height: 8,
    })?;
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
        builder.add_glyph(glyph);
    }
    builder.build_to("./examples/bitkodi/dist")?;
    println!("ok, written well");
    Ok(())
}
