use std::{borrow::BorrowMut, collections::BTreeSet, fs, io, path::Path};

use jiff::{civil::date, tz::TimeZone, Timestamp, Unit};
use snafu::prelude::*;
use write_fonts::{
    from_obj::ToOwnedTable,
    read::{FontRef, TableProvider},
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
    types::{Fixed, LongDateTime, NameId, Tag},
    BuilderError, FontBuilder, OffsetMarker,
};
use yaff::GlyphDefinition;

use crate::glyph::BitmapMatrix;

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
    #[snafu(transparent)]
    Jiff { source: jiff::Error },
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
        // Apple requires `unitsPerEm` not to be less than 64.
        let size_multiplier = (64f64 / (options.height as f64)).ceil() as u16;
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
        self.matrices.push(BitmapMatrix::from(glyph));
    }

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err> {
        let (loca_format, num_glyphs, (glyf, loca, cmap, hmtx)) =
            self.make_glyph_related_tables()?;
        let hhea = self.make_hhea(&hmtx);
        let head = self.make_head(loca_format)?;
        let maxp = self.make_maxp(num_glyphs);
        let os2 = self.make_os2();
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
        let checksum = bytes
            .chunks(4)
            .map(|chunk| {
                let mut bytes = [0u8; 4];
                bytes[..chunk.len()].copy_from_slice(chunk);
                u32::from_be_bytes(bytes)
            })
            .fold(0u32, |acc, word| acc.wrapping_add(word));
        let checksum_adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum);

        eprintln!(
            "checksum=0x{:X}, adjustment=0x{:X}",
            checksum, checksum_adjustment
        );

        let font = FontRef::new(&bytes).expect("fresh font must be parsed");
        let mut head: Head = font
            .head()
            .expect("head table must be exists")
            .to_owned_table();
        head.checksum_adjustment = checksum_adjustment;

        let bytes = FontBuilder::new()
            .add_table(&head)?
            .copy_missing_tables(font)
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
            (&now - &base).total(Unit::Second)? as i64
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

    fn make_hhea(&self, hmtx: &Hmtx /* vmtx: &Vmtx */) -> Hhea {
        Hhea {
            ascender: Default::default(),
            descender: Default::default(),
            line_gap: Default::default(),
            advance_width_max: Default::default(),
            min_left_side_bearing: Default::default(),
            min_right_side_bearing: Default::default(),
            x_max_extent: Default::default(),
            caret_slope_rise: Default::default(),
            caret_slope_run: Default::default(),
            caret_offset: Default::default(),
            number_of_long_metrics: hmtx.h_metrics.len() as _, /* + vmtx.v_metrics.len() */
        }
    }

    fn make_maxp(&self, num_glyphs: u16) -> Maxp {
        Maxp::new(num_glyphs)
    }

    fn make_os2(&self) -> Os2 {
        Os2::default()
    }

    fn make_glyph_related_tables(
        &self,
    ) -> Result<(LocaFormat, u16, (Glyf, Loca, Cmap, Hmtx)), OpentypeTtfBuildError> {
        let mut num_glyphs = 0;
        let mut hmtx_h_metrics = Vec::new();
        let mut hmtx_left_side_bearings = Vec::new();

        let mut glyf_loca_builder = GlyfLocaBuilder::new();

        for matrix in &self.matrices {
            let (paths, bb) = matrix.as_bezier_paths(self.size_multiplier as _);
            match &paths[..] {
                [path] => {
                    glyf_loca_builder.add_glyph(
                        &SimpleGlyph::from_bezpath(path).expect("must be valid bezier path"),
                    )?;
                    hmtx_h_metrics.push(LongMetric::new(
                        bb.width().try_into().expect("width <= 16 i believe"),
                        0,
                    ));
                    hmtx_left_side_bearings.push(0);
                    num_glyphs += 1;
                }
                _ => {
                    eprintln!("there is unsupported glyph");
                }
            }
        }

        let (glyf, loca, loca_format) = glyf_loca_builder.build();
        let hmtx = Hmtx::new(hmtx_h_metrics, hmtx_left_side_bearings);
        let cmap = Cmap::default();

        Ok((loca_format, num_glyphs, (glyf, loca, cmap, hmtx)))
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
