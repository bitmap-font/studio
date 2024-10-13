# yaff

Implementation for [monobit yaff](https://github.com/robhagemans/monobit/blob/master/YAFF.md) in Rust.

## Differences to the Spec

- "A yaff file must not contain control characters, other than the ones mentioned above, or UTF-8 noncharacters." is not enforced.
- Deprecated plain label is not supported due to its ambiguity to the property syntax.
- It is not strictly ensure the glyph data is inked or not; `[0-9A-F]` can be used inside glyph so it can be color-font. `@` is considered as `0`th color (`.` is 17-th transparent index or simply `None`).
- It allows at most one whitespace between every glyph character (the whitespace will be ignored). because since the ascii glyphs are normally half-width in duospaced fonts, giving it a gap makes the glyphs render better in text editors.
  <details>
  <summary>See example</summary>

  ```
   Too narrow | Looks great
  ------------+-------------
     .@..     |   . @ . .
     @.@.     |   @ . @ .
     @@@.     |   @ @ @ .
     @.@.     |   @ . @ .
     @.@.     |   @ . @ .
  ```

## Limitations

- Currently the yaff crate focuses on reading yaff files, no write support at this time.
  - We're very interested in format-preserving modification of yaff document see you soon!
