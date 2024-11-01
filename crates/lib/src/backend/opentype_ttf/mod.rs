use std::{collections::HashMap, fs, io, path::Path, time::Instant};

use jiff::{
    civil::{date, datetime, DateTime, Time},
    tz::TimeZone,
    Timestamp,
};
use snafu::prelude::*;
use write_fonts::{
    tables::{
        cmap::Cmap,
        glyf::{Glyf, GlyfLocaBuilder},
        head::{Head, MacStyle},
        hhea::Hhea,
        hmtx::Hmtx,
        loca::{Loca, LocaFormat},
        maxp::Maxp,
        name::Name,
        os2::Os2,
        post::Post,
        sbix::HeaderFlags,
    },
    types::{FWord, Fixed, LongDateTime, Tag, UfWord},
    BuilderError, FontBuilder,
};
use yaff::{GlyphDefinition, SemanticGlyphLabel};

use super::{FontBackend, FontOptions};

pub struct OpentypeTtfBackend {
    options: FontOptions,
    max_width: u16,
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
}

impl FontBackend for OpentypeTtfBackend {
    type Err = OpentypeTtfBuildError;

    fn new(options: FontOptions) -> Self {
        OpentypeTtfBackend {
            options,
            max_width: 0,
        }
    }

    fn add_glyphs(&mut self, map: HashMap<SemanticGlyphLabel, GlyphDefinition>) {
        todo!()
    }

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err> {
        let (glyf, loca, loca_format) = self.make_glyf_loca();
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
        let checksum_adjustment: u32 = {
            let sum: u32 = bytes.into_iter().map(|v| v as u32).sum();
            0xB1B0AFBAu32 - sum
        };

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
        fs::write(dir.join(format!("{}.ttf", self.options.name)), bytes)?;

        Ok(())
    }
}

impl OpentypeTtfBackend {
    fn make_head(&self, loca_format: LocaFormat) -> Result<Head, OpentypeTtfBuildError> {
        if self.options.height == 0 {
            return Err(OpentypeTtfBuildError::FontHeightZero);
        }
        if self.options.height > 16384 {
            return Err(OpentypeTtfBuildError::FontHeightTooBig {
                height: self.options.height,
            });
        }
        let size_multiplier = (16f64 / (self.options.height as f64)).ceil() as u16;

        let time = {
            let base = date(1904, 1, 1)
                .to_zoned(TimeZone::UTC)
                .expect("1904-01-01T00:00:00Z must be presentable in timestamp");
            let now = Timestamp::now().to_zoned(TimeZone::UTC);
            (&now - &base).get_seconds()
        };
        Ok(Head {
            font_revision: Fixed::from_f64(self.options.revision),

            checksum_adjustment: 0,
            magic_number: 0x5F0F3CF5,

            flags: HeaderFlags::empty().bits(),

            units_per_em: self.options.height * size_multiplier,

            created: LongDateTime::new(time),
            modified: LongDateTime::new(time),

            // @TODO i'm not confident about this
            x_min: 0,
            y_min: 0,
            x_max: (self.max_width * size_multiplier) as i16,
            y_max: (self.options.height * size_multiplier) as i16,

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
        Maxp::default()
    }

    fn make_os2(&self) -> Os2 {
        Os2::default()
    }

    fn make_hmtx(&self) -> Hmtx {
        Hmtx::default()
    }

    fn make_cmap(&self) -> Cmap {
        Cmap::default()
    }

    fn make_glyf_loca(&self) -> (Glyf, Loca, LocaFormat) {
        let mut builder = GlyfLocaBuilder::new();

        builder.build()
    }

    fn make_name(&self) -> Name {
        Name::default()
    }

    fn make_post(&self) -> Post {
        Post::default()
    }
}
