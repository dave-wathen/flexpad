use std::str::from_utf8_unchecked;

pub struct Workpad;

pub struct SheetCell {
    row: usize,
    column: usize,
}

pub struct Column {
    column: usize,
}

#[derive(Debug)]
pub enum CellReference {
    A1(u8, [u8; 27]),
}

impl Default for Workpad {
    fn default() -> Self {
        Self
    }
}

impl Workpad {
    pub fn column_header_height(&self) -> f32 {
        20.0
    }

    pub fn row_header_width(&self) -> f32 {
        60.0
    }

    pub fn row_count(&self) -> usize {
        1000
    }

    pub fn column_count(&self) -> usize {
        100
    }

    pub fn row_height(&self, _row: usize) -> f32 {
        20.0
    }

    pub fn column_width(&self, _column: usize) -> f32 {
        100.0
    }

    pub fn cell(&self, row: usize, column: usize) -> SheetCell {
        SheetCell::new(row, column)
    }

    pub fn column(&self, column: usize) -> Column {
        Column::new(column)
    }
}

impl SheetCell {
    fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn a1_reference(&self) -> CellReference {
        create_a1_reference(self.row, self.column)
    }
}

impl Column {
    fn new(column: usize) -> Self {
        Self { column }
    }

    // TODO Do we want a short string type that supports Into<WidgetText>? Once decided deal with duplicste code below
    pub fn name(&self) -> String {
        create_column_name(self.column)
    }
}

fn create_a1_reference(row: usize, column: usize) -> CellReference {
    // We build the reference backwards.
    // The row part is simple the number of the row so we accumulate each digit
    // in turn from the least significant.
    //
    // The columns are a series of base 26 (A=0, B=1, Z=25) number sequences
    // that are zero-(A-)padded to a length so one range is distinguished from
    // others.  That is:
    //    The first 26 are a one-digit sequence A-Z
    //    The next 26*26 are a two-digit sequence AA-ZZ
    //    The next 26*26*26 are a three-digit sequence AAA-ZZZ
    //    ...
    let mut start = 27;
    let mut bytes = [0 as u8; 27];
    let mut push = |b| {
        start -= 1;
        bytes[start] = b;
    };

    let mut rw = row + 1;
    loop {
        push('0' as u8 + (rw % 10) as u8);
        if rw < 10 {
            break;
        }
        rw /= 10;
    }

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
        LO_1..=HI_1 => push('A' as u8 + column as u8),
        LO_2..=HI_2 => {
            let cl = column - LO_2;
            push('A' as u8 + (cl % 26) as u8);
            push('A' as u8 + ((cl / 26) % 26) as u8);
        }
        LO_3..=HI_3 => {
            dbg!("LO_3..=HI_3");
            let cl = column - LO_3;
            push('A' as u8 + (cl % 26) as u8);
            push('A' as u8 + ((cl / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
        }
        LO_4..=HI_4 => {
            let cl = column - LO_4;
            push('A' as u8 + (cl % 26) as u8);
            push('A' as u8 + ((cl / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26 / 26) % 26) as u8);
        }
        LO_5..=HI_5 => {
            let cl = column - LO_5;
            push('A' as u8 + (cl % 26) as u8);
            push('A' as u8 + ((cl / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26 / 26) % 26) as u8);
            push('A' as u8 + ((cl / 26 / 26 / 26 / 26) % 26) as u8);
        }
        _ => panic!("Column too large"),
    };

    CellReference::A1(start as u8, bytes)
}

fn create_column_name(column: usize) -> String {
    // We build the name backwards.
    // The column names are a series of base 26 (A=0, B=1, ... Z=25) number sequences
    // that are zero-(A-)padded to a length so one range is distinguished from
    // others.  That is:
    //    The first 26 are a one-digit sequence A-Z
    //    The next 26*26 are a two-digit sequence AA-ZZ
    //    The next 26*26*26 are a three-digit sequence AAA-ZZZ
    //    ...
    let mut bytes = vec![];

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
        LO_1..=HI_1 => bytes.push('A' as u8 + column as u8),
        LO_2..=HI_2 => {
            let cl = column - LO_2;
            bytes.push('A' as u8 + (cl % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26) % 26) as u8);
        }
        LO_3..=HI_3 => {
            dbg!("LO_3..=HI_3");
            let cl = column - LO_3;
            bytes.push('A' as u8 + (cl % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
        }
        LO_4..=HI_4 => {
            let cl = column - LO_4;
            bytes.push('A' as u8 + (cl % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26 / 26) % 26) as u8);
        }
        LO_5..=HI_5 => {
            let cl = column - LO_5;
            bytes.push('A' as u8 + (cl % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26 / 26) % 26) as u8);
            bytes.push('A' as u8 + ((cl / 26 / 26 / 26 / 26) % 26) as u8);
        }
        _ => panic!("Column too large"),
    };

    bytes.reverse();
    unsafe { String::from_utf8_unchecked(bytes) }
}

impl std::ops::Deref for CellReference {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            CellReference::A1(start, ref bytes) => unsafe {
                from_utf8_unchecked(&bytes[(*start as usize)..27])
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_a1_references() {
        assert_eq!("A1", &create_a1_reference(0, 0) as &str);
        assert_eq!("A9", &create_a1_reference(8, 0) as &str);
        assert_eq!("A10", &create_a1_reference(9, 0) as &str);
        assert_eq!("A1234567", &create_a1_reference(1234566, 0) as &str);

        assert_eq!("B1", &create_a1_reference(0, 1) as &str);
        assert_eq!("Z1", &create_a1_reference(0, 25) as &str);
        assert_eq!("AA1", &create_a1_reference(0, 26) as &str);
        assert_eq!("AZ1", &create_a1_reference(0, 51) as &str);
        assert_eq!("BA1", &create_a1_reference(0, 52) as &str);
        assert_eq!("ZZ1", &create_a1_reference(0, 26_usize.pow(2) + 25) as &str);
        assert_eq!(
            "AAA1",
            &create_a1_reference(0, 26_usize.pow(2) + 26) as &str
        );
        assert_eq!(
            "ZZZ1",
            &create_a1_reference(0, 26_usize.pow(3) + 26_usize.pow(2) + 25) as &str
        );
        assert_eq!(
            "AAAA1",
            &create_a1_reference(0, 26_usize.pow(3) + 26_usize.pow(2) + 26) as &str
        );
        assert_eq!(
            "ZZZZ1",
            &create_a1_reference(0, 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 25)
                as &str
        );
        assert_eq!(
            "AAAAA1",
            &create_a1_reference(0, 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 26)
                as &str
        );
        assert_eq!(
            "ZZZZZ1",
            &create_a1_reference(
                0,
                26_usize.pow(5) + 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 25
            ) as &str
        );
    }

    #[test]
    fn column_name() {
        assert_eq!("A", &create_column_name(0));
        assert_eq!("B", &create_column_name(1));
        assert_eq!("Z", &create_column_name(25));
        assert_eq!("AA", &create_column_name(26));
        assert_eq!("AZ", &create_column_name(51));
        assert_eq!("BA", &create_column_name(52));
        assert_eq!("ZZ", &create_column_name(26_usize.pow(2) + 25));
        assert_eq!("AAA", &create_column_name(26_usize.pow(2) + 26));
        assert_eq!(
            "ZZZ",
            &create_column_name(26_usize.pow(3) + 26_usize.pow(2) + 25)
        );
        assert_eq!(
            "AAAA",
            &create_column_name(26_usize.pow(3) + 26_usize.pow(2) + 26)
        );
        assert_eq!(
            "ZZZZ",
            &create_column_name(26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 25)
        );
        assert_eq!(
            "AAAAA",
            &create_column_name(26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 26)
        );
        assert_eq!(
            "ZZZZZ",
            &create_column_name(
                26_usize.pow(5) + 26_usize.pow(4) + 26_usize.pow(3) + 26_usize.pow(2) + 25
            )
        );
    }
}
