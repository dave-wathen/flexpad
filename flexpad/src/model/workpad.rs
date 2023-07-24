use compact_str::{CompactString, ToCompactString};

pub struct Workpad {
    name: CompactString,
    sheets: Vec<SheetData>,
    current: usize,
}

impl Default for Workpad {
    fn default() -> Self {
        let sheet1 = SheetData::new("Sheet 1");
        let sheet2 = SheetData::new("Sheet 2");
        let sheet3 = SheetData::new("Sheet 3");
        Self {
            name: "Unnamed".to_compact_string(),
            sheets: vec![sheet1, sheet2, sheet3],
            current: 0,
        }
    }
}

impl Workpad {
    pub fn active_sheet(&self) -> Sheet<'_> {
        Sheet {
            data: &self.sheets[self.current],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // TODO Think about MVCC
    pub fn set_name(&mut self, value: impl ToCompactString) {
        self.name = value.to_compact_string()
    }
}

pub struct SheetData {
    name: CompactString,
    column_header_height: f32,
    row_header_width: f32,
    columns: Vec<ColumnData>,
    rows: Vec<RowData>,
}

impl SheetData {
    fn new(name: impl ToCompactString) -> Self {
        // TODO Temporarilly overridden until scroll window only is working
        let columns = (0..99).map(ColumnData::new).collect();
        let rows = (0..999).map(RowData::new).collect();
        // let columns = (0..30).map(ColumnData::new).collect();
        // let rows = (0..99).map(RowData::new).collect();
        Self {
            name: name.to_compact_string(),
            column_header_height: 20.0,
            row_header_width: 60.0,
            columns,
            rows,
        }
    }
}

pub struct ColumnData {
    name: Name,
    width: f32,
}

impl ColumnData {
    fn new(index: usize) -> Self {
        Self {
            name: Name::Auto(create_column_name(index)),
            width: 100.0,
        }
    }
}

pub struct RowData {
    name: Name,
    height: f32,
}

impl RowData {
    fn new(index: usize) -> Self {
        Self {
            name: Name::Auto((index + 1).to_compact_string()),
            height: 20.0,
        }
    }
}

pub struct Sheet<'pad> {
    data: &'pad SheetData,
}

impl Sheet<'_> {
    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn columns(&self) -> impl ExactSizeIterator<Item = Column<'_>> {
        self.data
            .columns
            .iter()
            .enumerate()
            .map(|(index, data)| Column::new(data, index))
    }

    #[allow(dead_code)]
    pub fn column(&self, index: usize) -> Column<'_> {
        Column::new(&self.data.columns[index], index)
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = Row<'_>> {
        self.data
            .rows
            .iter()
            .enumerate()
            .map(|(index, data)| Row::new(data, index))
    }

    #[allow(dead_code)]
    pub fn row(&self, index: usize) -> Row<'_> {
        Row::new(&self.data.rows[index], index)
    }

    pub fn column_header_height(&self) -> f32 {
        self.data.column_header_height
    }

    pub fn row_header_width(&self) -> f32 {
        self.data.row_header_width
    }

    pub fn active_cell(&self) -> Cell<'_> {
        // TODO
        self.cell(0, 0)
    }

    pub fn cells(&self) -> impl Iterator<Item = Cell<'_>> {
        // TODO use a range
        let rows = self.data.rows.len();
        let cols = self.data.columns.len();
        let mut rw = 0;
        let mut cl = 0;
        std::iter::from_fn(move || {
            if rw >= rows {
                None
            } else {
                let cell = self.cell(rw, cl);
                cl += 1;
                if cl >= cols {
                    cl = 0;
                    rw += 1;
                }
                Some(cell)
            }
        })
    }

    pub fn cell(&self, row: usize, column: usize) -> Cell<'_> {
        Cell::new(
            &self.data.rows[row],
            row,
            &self.data.columns[column],
            column,
        )
    }
}

pub struct Column<'pad> {
    data: &'pad ColumnData,
    index: usize,
}

impl Column<'_> {
    fn new(data: &'_ ColumnData, index: usize) -> Column<'_> {
        Column { data, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }

    pub fn width(&self) -> f32 {
        self.data.width
    }
}

pub struct Row<'pad> {
    data: &'pad RowData,
    index: usize,
}

impl Row<'_> {
    fn new(data: &'_ RowData, index: usize) -> Row<'_> {
        Row { data, index }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }

    pub fn height(&self) -> f32 {
        self.data.height
    }
}

pub struct Cell<'pad> {
    row_data: &'pad RowData,
    row_index: usize,
    column_data: &'pad ColumnData,
    column_index: usize,
    name: Name,
}

#[allow(dead_code)]
impl Cell<'_> {
    fn new<'pad>(
        row_data: &'pad RowData,
        row_index: usize,
        column_data: &'pad ColumnData,
        column_index: usize,
    ) -> Cell<'pad> {
        let name = match (&row_data.name, &column_data.name) {
            (Name::Auto(row_name), Name::Auto(column_name)) => {
                Name::Auto(column_name.clone() + row_name)
            }
            (Name::Auto(_), Name::Custom(_)) => todo!(),
            (Name::Custom(_), Name::Auto(_)) => todo!(),
            (Name::Custom(_), Name::Custom(_)) => todo!(),
        };
        Cell {
            row_data,
            row_index,
            column_data,
            column_index,
            name,
        }
    }

    pub fn row(&self) -> Row<'_> {
        Row::new(self.row_data, self.row_index)
    }

    pub fn column(&self) -> Column<'_> {
        Column::new(self.column_data, self.column_index)
    }

    pub fn width(&self) -> f32 {
        self.column_data.width
    }

    pub fn height(&self) -> f32 {
        self.row_data.height
    }

    pub fn name(&self) -> &str {
        match &self.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }
}

#[allow(dead_code)]
enum Name {
    Auto(CompactString),
    Custom(CompactString),
}

fn create_column_name(column: usize) -> CompactString {
    // The column names are a series of base 26 (A=0, B=1, ... Z=25) number sequences
    // that are zero-(A-)padded to a length so one range is distinguished from
    // others.  That is:
    //    The first 26 are a one-digit sequence A-Z
    //    The next 26*26 are a two-digit sequence AA-ZZ
    //    The next 26*26*26 are a three-digit sequence AAA-ZZZ
    //    ...
    const LO_1: usize = 0;
    const HI_1: usize = 25;
    const LO_2: usize = HI_1 + 1;
    const HI_2: usize = HI_1 + 26_usize.pow(2);
    const LO_3: usize = HI_2 + 1;
    const HI_3: usize = HI_2 + 26_usize.pow(3);
    const LO_4: usize = HI_3 + 1;
    const HI_4: usize = HI_3 + 26_usize.pow(4);
    const LO_5: usize = HI_4 + 1;
    const HI_5: usize = HI_4 + 26_usize.pow(5);

    match column {
        LO_1..=HI_1 => unsafe { CompactString::from_utf8_unchecked([b'A' + column as u8]) },
        LO_2..=HI_2 => {
            let cl = column - LO_2;
            unsafe {
                CompactString::from_utf8_unchecked([
                    b'A' + ((cl / 26) % 26) as u8,
                    b'A' + (cl % 26) as u8,
                ])
            }
        }
        LO_3..=HI_3 => {
            let cl = column - LO_3;
            unsafe {
                CompactString::from_utf8_unchecked([
                    b'A' + ((cl / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26) % 26) as u8,
                    b'A' + (cl % 26) as u8,
                ])
            }
        }
        LO_4..=HI_4 => {
            let cl = column - LO_4;
            unsafe {
                CompactString::from_utf8_unchecked([
                    b'A' + ((cl / 26 / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26) % 26) as u8,
                    b'A' + (cl % 26) as u8,
                ])
            }
        }
        LO_5..=HI_5 => {
            let cl = column - LO_5;
            unsafe {
                CompactString::from_utf8_unchecked([
                    b'A' + ((cl / 26 / 26 / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26 / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26 / 26) % 26) as u8,
                    b'A' + ((cl / 26) % 26) as u8,
                    b'A' + (cl % 26) as u8,
                ])
            }
        }
        _ => panic!("Column too large"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_name() {
        const A: usize = 0;
        const AA: usize = 26;
        const AAA: usize = 26_usize.pow(2) + 26;
        const AAAA: usize = 26_usize.pow(3) + 26_usize.pow(2) + 26;
        const AAAAA: usize = 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 26;
        const AAAAAA: usize =
            26_usize.pow(5) + 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 26;

        assert_eq!("A", &create_column_name(A) as &str);
        assert_eq!("B", &create_column_name(A + 1) as &str);
        assert_eq!("Z", &create_column_name(AA - 1) as &str);
        assert_eq!("AA", &create_column_name(AA) as &str);
        assert_eq!("AB", &create_column_name(AA + 1) as &str);
        assert_eq!("AZ", &create_column_name(AA + 25) as &str);
        assert_eq!("BA", &create_column_name(AA + 26) as &str);
        assert_eq!("ZZ", &create_column_name(AAA - 1) as &str);
        assert_eq!("AAA", &create_column_name(AAA) as &str);
        assert_eq!("ABC", &create_column_name(AAA + 26 + 2) as &str);
        assert_eq!("ZZZ", &create_column_name(AAAA - 1) as &str);
        assert_eq!("AAAA", &create_column_name(AAAA) as &str);
        assert_eq!(
            "ABCD",
            &create_column_name(AAAA + 26_usize.pow(2) + (2 * 26) + 3) as &str
        );
        assert_eq!("ZZZZ", &create_column_name(AAAAA - 1) as &str);
        assert_eq!("AAAAA", &create_column_name(AAAAA) as &str);
        assert_eq!(
            "ABCDE",
            &create_column_name(AAAAA + 26_usize.pow(3) + (2 * 26_usize.pow(2)) + (3 * 26) + 4)
                as &str
        );
        assert_eq!("ZZZZZ", &create_column_name(AAAAAA - 1) as &str);
    }
}
