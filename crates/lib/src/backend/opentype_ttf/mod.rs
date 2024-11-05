use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

use jiff::{civil::date, tz::TimeZone, Timestamp, Unit};
use snafu::prelude::*;
use write_fonts::{
    from_obj::ToOwnedTable,
    read::{FontRef, TableProvider},
    tables::{
        cmap::{Cmap, CmapSubtable, EncodingRecord, PlatformId, SequentialMapGroup},
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
    types::{FWord, Fixed, LongDateTime, NameId},
    BuilderError, FontBuilder, OffsetMarker,
};
use yaff::{GlyphDefinition, SemanticGlyphLabel};

use crate::glyph::{
    path::{analyze_bezpath, PointAndContours},
    BitmapMatrix,
};

use super::{FontBackend, FontOptions};

pub struct OpentypeTtfBackend {
    options: FontOptions,
    size_multiplier: u16,
    max_width: u16,
    matrices: Vec<(Vec<SemanticGlyphLabel>, BitmapMatrix)>,
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
        self.max_width = self.max_width.max(glyph_value.width);
        dbg!(BitmapMatrix::from(glyph).as_bezier_paths(1).0[0].to_svg());
        self.matrices.push((
            glyph
                .labels
                .iter()
                .map(|label| label.to_semantic())
                .flatten()
                .collect(),
            BitmapMatrix::from(glyph),
        ));
    }

    fn build_to(self, dir: impl AsRef<Path>) -> Result<(), Self::Err> {
        let (loca_format, (glyf, loca, cmap, hmtx, maxp)) = self.make_glyph_related_tables()?;
        let hhea = self.make_hhea(&hmtx);
        let head = self.make_head(loca_format)?;
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
                bytes.copy_from_slice(chunk);
                u32::from_be_bytes(bytes)
            })
            .fold(0u32, |acc, word| acc.wrapping_add(word));
        // let checksum = 0;
        // let checksum_adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum);
        let checksum_adjustment = 0;

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
            x_max: (self.max_width * self.size_multiplier) as _,
            y_max: (self.options.height * self.size_multiplier) as _,

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
            ascender: FWord::new((self.options.ascender * self.size_multiplier) as _),
            descender: FWord::new(-((self.options.descender * self.size_multiplier) as i16)),
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

    fn make_os2(&self) -> Os2 {
        Os2 {
            x_avg_char_width: Default::default(),
            // @TODO change this
            us_weight_class: 400,
            us_width_class: 5,
            fs_type: Default::default(),
            y_subscript_x_size: Default::default(),
            y_subscript_y_size: Default::default(),
            y_subscript_x_offset: Default::default(),
            y_subscript_y_offset: Default::default(),
            y_superscript_x_size: Default::default(),
            y_superscript_y_size: Default::default(),
            y_superscript_x_offset: Default::default(),
            y_superscript_y_offset: Default::default(),
            y_strikeout_size: Default::default(),
            y_strikeout_position: Default::default(),
            s_family_class: Default::default(),
            panose_10: Default::default(),
            ul_unicode_range_1: Default::default(),
            ul_unicode_range_2: Default::default(),
            ul_unicode_range_3: Default::default(),
            ul_unicode_range_4: Default::default(),
            ach_vend_id: Default::default(),
            fs_selection: Default::default(),
            us_first_char_index: Default::default(),
            us_last_char_index: Default::default(),
            s_typo_ascender: (self.options.ascender * self.size_multiplier) as _,
            s_typo_descender: -((self.options.descender * self.size_multiplier) as i16),
            s_typo_line_gap: Default::default(),
            us_win_ascent: Default::default(),
            us_win_descent: Default::default(),
            ul_code_page_range_1: Default::default(),
            ul_code_page_range_2: Default::default(),
            sx_height: Default::default(),
            s_cap_height: Default::default(),
            us_default_char: Default::default(),
            us_break_char: Default::default(),
            us_max_context: Default::default(),
            us_lower_optical_point_size: Default::default(),
            us_upper_optical_point_size: Default::default(),
        }
    }

    fn make_glyph_related_tables(
        &self,
    ) -> Result<(LocaFormat, (Glyf, Loca, Cmap, Hmtx, Maxp)), OpentypeTtfBuildError> {
        let mut num_glyphs = 0u16;
        let mut max_points = 0u16;
        let mut max_contours = 0u16;

        let mut hmtx_h_metrics = Vec::new();
        let mut hmtx_left_side_bearings = Vec::new();

        let mut character_mappings = BTreeMap::new();

        let mut glyf_loca_builder = GlyfLocaBuilder::new();

        for (labels, matrix) in &self.matrices {
            let mut groups = Vec::new();
            for label in labels {
                match label {
                    SemanticGlyphLabel::CharSequence(vec) => match &vec[..] {
                        &[ch] => groups.push(ch),
                        _ => {
                            eprintln!("{} is not supported yet", vec.iter().collect::<String>());
                            continue;
                        }
                    },
                    SemanticGlyphLabel::Tag(tag) => {
                        let Some(ch) = unicode_names2::character(&tag) else {
                            continue;
                        };
                        groups.push(ch);
                    }
                }
            }

            let (paths, bb) = matrix.as_bezier_paths(self.size_multiplier as _);
            match &paths[..] {
                [path] => {
                    glyf_loca_builder.add_glyph(
                        &SimpleGlyph::from_bezpath(path).expect("must be valid bezier path"),
                    )?;
                    hmtx_h_metrics.push(LongMetric::new(bb.width() as _, 0));
                    hmtx_left_side_bearings.push(0);
                    let PointAndContours { points, contours } = analyze_bezpath(path);
                    max_points = max_points.max(points as _);
                    max_contours = max_contours.max(contours as _);

                    for ch in groups {
                        character_mappings.insert(ch, num_glyphs);
                    }

                    num_glyphs += 1;
                }
                _ => {
                    eprintln!("there is unsupported glyph");
                }
            }
        }

        let (glyf, loca, loca_format) = glyf_loca_builder.build();
        let hmtx = Hmtx::new(hmtx_h_metrics, hmtx_left_side_bearings);
        let cmap = Cmap::new(vec![{
            let mut groups = Vec::new();
            for (ch, id) in character_mappings.into_iter() {
                groups.push(SequentialMapGroup::new(ch as _, ch as _, id as _));
            }

            EncodingRecord::new(PlatformId::Unicode, 6, CmapSubtable::format_12(
                // header = u16 + u16
                4 + 
                // length = u32
                4 + 
                // language = u32
                4 + 
                // num_groups = u32
                4 +
                // groups = {num_groups} * (u32 + u32 + u32)
                12 * groups.len() as u32, 
                // The language field must be set to zero for all 'cmap' subtables whose platform IDs are other than Macintosh (platform ID 1)
                0, groups.len() as _, groups))
        }]);

        let maxp = Maxp {
            num_glyphs,
            // since we don't use quad or curve, it's identical.
            max_points: Some(max_points),
            max_contours: Some(max_contours),

            // no composite glyph for now.
            max_composite_points: Some(0),
            max_composite_contours: Some(0),
            max_component_elements: Some(0),
            max_component_depth: Some(1),

            // we don't use twilight zone for now to reduce complexity.
            // hinting is very very hard problem.
            max_zones: Some(1),
            max_twilight_points: Some(0),
            max_storage: Some(0),
            max_function_defs: Some(0),
            max_instruction_defs: Some(0),
            max_size_of_instructions: Some(0),

            max_stack_elements: Some(1),
        };

        Ok((loca_format, (glyf, loca, cmap, hmtx, maxp)))
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
