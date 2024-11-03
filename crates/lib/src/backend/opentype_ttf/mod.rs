use std::{collections::BTreeSet, fs, io, iter::once, path::Path};

use jiff::{civil::date, tz::TimeZone, Timestamp};
use snafu::prelude::*;
use write_fonts::{
    tables::{
        cmap::{Cmap, PlatformId},
        glyf::{Glyf, GlyfLocaBuilder, SimpleGlyph},
        head::{Head, MacStyle},
        hhea::Hhea,
        hmtx::Hmtx,
        loca::{Loca, LocaFormat},
        maxp::Maxp,
        name::{Name, NameRecord},
        os2::Os2,
        post::Post,
        sbix::HeaderFlags,
        vmtx::LongMetric,
    },
    types::{Fixed, LongDateTime, NameId},
    BuilderError, FontBuilder, OffsetMarker,
};
use yaff::GlyphDefinition;

use crate::bitmap_matrix::BitmapMatrix;

use super::{FontBackend, FontOptions};

pub struct OpentypeTtfBackend {
    options: FontOptions,
    size_multiplier: u16,
    max_width: u16,
    matrices: Vec<BitmapMatrix>,
}

#[derive(Debug, Snafu)]
pub enum OpentypeTtfBuildError {
    #[snafu(display("font height must not be zero"))]
    FontHeightZero,
    #[snafu(display("expect font-height <= 16384 but got {height}"))]
    FontHeightTooBig { height: u16 },
    #[snafu(transparent)]
    Builder { source: BuilderError },
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(transparent)]
    WriteFonts { source: write_fonts::error::Error },
}

impl OpentypeTtfBackend {
    pub fn new(options: FontOptions) -> Result<Self, OpentypeTtfBuildError> {
        if options.height == 0 {
            return Err(OpentypeTtfBuildError::FontHeightZero);
        }
        if options.height > 16384 {
            return Err(OpentypeTtfBuildError::FontHeightTooBig {
                height: options.height,
            });
        }
        let size_multiplier = (16f64 / (options.height as f64)).ceil() as u16;
        Ok(OpentypeTtfBackend {
            options,
            size_multiplier,
            max_width: 0,
            matrices: Vec::new(),
        })
    }
}

impl FontBackend for OpentypeTtfBackend {
    type Err = OpentypeTtfBuildError;

    fn add_glyph(&mut self, glyph: &GlyphDefinition) {
        let Some(glyph_value) = &glyph.value else {
            return;
        };
        // match label {
        //     SemanticGlyphLabel::CharSequence(vec) => {
        //         if vec.len() != 1 {
        //             panic!("not supported for now")
        //         } else {
        //         }
        //     }
        //     SemanticGlyphLabel::Tag(tag) => todo!(),
        // }
        self.max_width = self.max_width.max(glyph_value.width);
        BitmapMatrix::from(glyph).as_bezier_paths();
        self.matrices.push(BitmapMatrix::from(glyph));
    }

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err> {
        let (glyf, loca, loca_format) = self.make_glyf_loca()?;
        let cmap = self.make_cmap();
        let hhea = self.make_hhea();
        let head = self.make_head(loca_format)?;
        let maxp = self.make_maxp();
        let os2 = self.make_os2();
        let hmtx = self.make_hmtx();
        let name = self.make_name();
        let post = self.make_post();

        // write_fonts does not calculate checksum for now.
        let bytes = FontBuilder::new()
            .add_table(&head)?
            .add_table(&hhea)?
            .add_table(&maxp)?
            .add_table(&os2)?
            .add_table(&hmtx)?
            .add_table(&cmap)?
            .add_table(&loca)?
            .add_table(&glyf)?
            .add_table(&name)?
            .add_table(&post)?
            .build();
        let checksum_adjustment = {
            let sum = bytes
                .chunks(4)
                .into_iter()
                .map(|v| {
                    let mut bytes = [0u8; 4];
                    bytes[..v.len()].copy_from_slice(v);
                    u32::from_be_bytes(bytes)
                })
                .fold(0u32, |acc, word| acc.wrapping_add(word));
            0xB1B0AFBAu32.wrapping_sub(sum)
        };
        eprintln!("checksum=0x{:X}", checksum_adjustment);

        let bytes = FontBuilder::new()
            .add_table(&Head {
                checksum_adjustment,
                ..head
            })?
            .add_table(&hhea)?
            .add_table(&maxp)?
            .add_table(&os2)?
            .add_table(&hmtx)?
            .add_table(&cmap)?
            .add_table(&loca)?
            .add_table(&glyf)?
            .add_table(&name)?
            .add_table(&post)?
            .build();

        let dir = dir.as_ref();
        fs::remove_dir_all(dir)?;
        fs::create_dir_all(dir)?;
        fs::write(
            dir.join(format!(
                "{} {}.ttf",
                self.options.family_name, self.options.sub_family_name
            )),
            bytes,
        )?;

        Ok(())
    }
}

impl OpentypeTtfBackend {
    fn make_head(&self, loca_format: LocaFormat) -> Result<Head, OpentypeTtfBuildError> {
        let time = {
            let base = date(1904, 1, 1)
                .to_zoned(TimeZone::UTC)
                .expect("1904-01-01T00:00:00Z must be presentable in timestamp");
            let now = Timestamp::now().to_zoned(TimeZone::UTC);
            (&now - &base).get_seconds()
        };
        Ok(Head {
            font_revision: Fixed::from_f64(
                (self.options.version.major as f64) + (self.options.version.minor as f64) / 100.0,
            ),

            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,

            flags: HeaderFlags::empty().bits(),

            units_per_em: self.options.height * self.size_multiplier,

            created: LongDateTime::new(time),
            modified: LongDateTime::new(time),

            // @TODO i'm not confident about this
            x_min: 0,
            y_min: 0,
            x_max: (self.max_width * self.size_multiplier) as i16,
            y_max: (self.options.height * self.size_multiplier) as i16,

            // @TODO bold and italic support
            mac_style: MacStyle::empty(),
            lowest_rec_ppem: self.options.height,
            // deprecated in spec; set to 2
            font_direction_hint: 2,

            index_to_loc_format: loca_format as _,
        })
    }

    fn make_hhea(&self) -> Hhea {
        Hhea::default()
    }

    fn make_maxp(&self) -> Maxp {
        Maxp::new(self.matrices.len() as u16)
    }

    fn make_os2(&self) -> Os2 {
        Os2::default()
    }

    fn make_hmtx(&self) -> Hmtx {
        // @TODO refactor
        Hmtx {
            h_metrics: vec![LongMetric::new(
                self.options.height * self.size_multiplier,
                0,
            )],
            left_side_bearings: vec![],
        }
    }

    fn make_cmap(&self) -> Cmap {
        Cmap::default()
    }

    fn make_glyf_loca(&self) -> Result<(Glyf, Loca, LocaFormat), OpentypeTtfBuildError> {
        let mut builder = GlyfLocaBuilder::new();

        for matrix in &self.matrices {
            let paths = matrix.as_bezier_paths();
            match &paths[..] {
                [path] => {
                    builder.add_glyph(
                        &SimpleGlyph::from_bezpath(path).expect("must be valid bezier path"),
                    )?;
                }
                _ => {
                    eprintln!("there is unsupported glyph");
                }
            }
        }

        Ok(builder.build())
    }

    fn make_name(&self) -> Name {
        fn make_name_record(id: NameId, value: impl AsRef<str>) -> NameRecord {
            NameRecord::new(
                PlatformId::Unicode as _,
                // Unicode Full Repertoire
                4,
                // There are no platform-specific language IDs defined for the Unicode platform.
                // Language ID = 0 may be used for Unicode-platform strings, but this does not indicate any particular language.
                // Language IDs greater than or equal to 0x8000 may be used together with language-tag records, as described above.
                0,
                id,
                OffsetMarker::new(value.as_ref().to_owned()),
            )
        }
        Name::new(BTreeSet::from_iter(
            vec![
                self.options
                    .copyright_notice
                    .as_ref()
                    .map(|value| make_name_record(NameId::COPYRIGHT_NOTICE, value)),
                Some(make_name_record(
                    NameId::FAMILY_NAME,
                    &self.options.family_name,
                )),
                Some(make_name_record(
                    NameId::SUBFAMILY_NAME,
                    &self.options.sub_family_name,
                )),
                Some(make_name_record(NameId::UNIQUE_ID, &self.options.unique_id)),
                Some(make_name_record(
                    NameId::FULL_NAME,
                    &self.options.full_font_name.clone().unwrap_or_else(|| {
                        format!(
                            "{} {}",
                            self.options.family_name, self.options.sub_family_name
                        )
                    }),
                )),
                Some(make_name_record(
                    NameId::VERSION_STRING,
                    format!(
                        "Version {}.{:03}{}",
                        self.options.version.major,
                        self.options.version.minor,
                        self.options
                            .version
                            .metadata
                            .as_ref()
                            .map_or_else(|| "".to_string(), |v| format!(" {v}"))
                    ),
                )),
                Some(make_name_record(
                    NameId::POSTSCRIPT_NAME,
                    self.options.postscript_name.clone().unwrap_or_else(|| {
                        format!(
                            "{}-{}",
                            self.options.family_name, self.options.sub_family_name
                        )
                    }),
                )),
            ]
            .into_iter()
            .flatten(),
        ))
    }

    fn make_post(&self) -> Post {
        Post::default()
    }
}
