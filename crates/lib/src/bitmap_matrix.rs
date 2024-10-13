pub struct BitmapMatrix(pub Vec<Vec<Option<u8>>>);

impl BitmapMatrix {
    pub fn union(list: impl IntoIterator<Item = BitmapMatrix>) -> BitmapMatrix {
        let mut this = Vec::new();

        for BitmapMatrix(other) in list {
            while this.len() < other.len() {
                this.push(Vec::new());
            }
            for (r, row) in other.into_iter().enumerate() {
                while this[r].len() < row.len() {
                    this[r].push(None);
                }
                for (c, col) in row.into_iter().enumerate() {
                    let Some(col) = col else { continue };
                    this[r][c].replace(col);
                }
            }
        }

        BitmapMatrix(this)
    }
}
