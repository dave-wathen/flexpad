use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    ops::RangeBounds,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
};

use compact_str::{CompactString, ToCompactString};
use uuid::Uuid;

// Overview of the Workpad Model
// =============================
//
// Workpads are represented by a complex set of structs that create an MVCC
// data structure that allows us to step backwards and forwards through the
// version history.
//
// The entry point is WorkpadMaster that provides access to all versions of
// the pad.  WorkpadMaster is backed by WorkpadMasterData which holds all the
// data (for all versions) of a workpad.
//
// Each part of a Workpad's structure (Sheets, Rows, Columns, Cells, ...) is
// identified by an ID type (SheetId, RowId, ColumnId, CellId, ...).  The details
// for each part is held in Data type (SheetData, RowData, ColumnData, CellData,
// ...). The Data types are alway allocated on the heap and are immutable (changes
// are effected by a new version with new Data values as needed).  The Data types
// do not include the ID of the part which allows common Data values to be shared.
// (For example rows/columns are often the same when created so all can point at the
// same value.)
//
// The association between Id values and Data values is held in indices that are part
// of WorkpadMasterData.  These indices are used to find the pointer (Arc) for the Data
// value that is appropriate for a given Id and versoion.
//
// Updates are handled by applying a WorkpadUpdate which generates a new version and
// makes that the active version. Updates are always based on top of the active version.
// The active version need not be the highest which means it is possible for dead
// versions to exist.  For example if we create versions 1, 2, 3, 4, then step the active
// version back to 2, an update will create version 5 based on version 2 making versions
// 3 and 4 redundant.
//
// A further set of types provide access to a version of the Workpad.  The entry point for
// these is a Workpad (see WorkpadMaster::active_version).  From Workpad types for the
// various parts (Sheet, Row, Column, Cell, ...) give the detailed view of the version.
//
// ??? Factory methods for WorkpadUpdate ???

// TODO Apply Id and version to Rows
// TODO Apply Id and version to Columns
// TODO Apply Id and version to Cells
// TODO Update thread with message channel?
// TODO Feed versions to UI as required to trigger redraws/updates?
//

/// The version of a workpad
type Version = u32;

/// The type which collectively represents all versions of a workpad
///
/// From this it is possible to apply [`WorkpadUpdate`]s to create new
/// versions based off the currently active version.
///
/// A [`Workpad`] can be obtained from [`WorkpadMaster::active_version()`]
/// to allow navigation through a particular version of the workpad.
#[derive(Clone)]
pub struct WorkpadMaster {
    data: Arc<WorkpadMasterData>,
}

impl WorkpadMaster {
    /// Create a new [`WorkpadMaster`] representing a new workpad with a single
    // initial version.
    pub fn new() -> Self {
        let mut master_data = WorkpadMasterData {
            id: Uuid::new_v4().simple().to_string(),
            history: RwLock::new(vec![HistoryEntry {
                prior_version: None,
                update: WorkpadUpdate::NewWorkpad,
            }]),
            active_version: RwLock::new(0),
            next_part_id: AtomicU32::new(0),
            workpad_idx: Default::default(),
            sheets_idx: Default::default(),
        };

        let sheet1_id = master_data.create_sheet(0, "Sheet 1");
        let sheet2_id = master_data.create_sheet(0, "Sheet 2");
        let sheet3_id = master_data.create_sheet(0, "Sheet 3");

        let workpad_data = WorkpadData {
            name: "Unnamed".to_compact_string(),
            sheets: vec![sheet1_id, sheet2_id, sheet3_id],
            active_sheet: sheet1_id,
        };
        master_data.write_workpad(Arc::new(workpad_data), 0);

        WorkpadMaster {
            data: Arc::new(master_data),
        }
    }

    /// Returns a [`Workpad`] that allows navigation through the data
    /// structures representing the currently active version of the
    /// workpad.
    pub fn active_version(&self) -> Workpad {
        let active_version = self.data.active_version();
        let workpad_data = self.data.read_workpad(active_version);
        Workpad::new(self.data.clone(), active_version, workpad_data)
    }

    /// Update the workpad by creating a new version with the supplied
    /// [`WorkpadUpdate`] applied.  The generated version becomes the
    /// active version of the workpad.
    pub fn update(&mut self, update: WorkpadUpdate) {
        let active_version = self.data.active_version();
        let new_version = {
            let mut history = self.data.history.write().unwrap();
            let new_version = history.len();
            history.push(HistoryEntry {
                prior_version: Some(active_version),
                update: update.clone(),
            });
            new_version as Version
        };

        match update {
            WorkpadUpdate::NewWorkpad => panic!("NewWorkpad not allowed for exisiting Workpad"),
            WorkpadUpdate::SetWorkpadName { new_name } => {
                let workpad_data = self.data.read_workpad(active_version);
                let new_workpad_data = WorkpadData {
                    name: new_name.to_compact_string(),
                    ..(*workpad_data).clone()
                };
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SetSheetName { sheet_id, new_name } => {
                let sheet_data = self.data.read_sheet(sheet_id, active_version);
                let new_sheet_data = SheetData {
                    name: new_name.to_compact_string(),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(sheet_id, Arc::new(new_sheet_data), new_version);
            }
            WorkpadUpdate::SetSheetCellValue {
                sheet_id,
                row,
                column,
                value,
            } => {
                // TODO Temporary approach
                let sheet_data = self.data.read_sheet(sheet_id, active_version);
                let mut new_sheet_data = (*sheet_data).clone();
                let cell_data = new_sheet_data
                    .cells
                    .entry((row, column))
                    .or_insert_with(Default::default);
                cell_data.value = Value::String(value.to_compact_string());
                self.data
                    .write_sheet(sheet_id, Arc::new(new_sheet_data), new_version);
            }
        }

        *self.data.active_version.write().unwrap() = new_version;
    }
}

/// A change that can be applied to a workpad to create a new version.
/// See [`WorkpadMaster::update(`)].
#[derive(Debug, Clone)]
pub enum WorkpadUpdate {
    /// Used to represent the creation of a workpad.  See [`WorkpadMaster::new`].
    NewWorkpad,
    /// Instruction to change the name of the workpad
    SetWorkpadName { new_name: String },
    /// Instruction to change the name of a specific sheet within a workpad.
    SetSheetName { sheet_id: SheetId, new_name: String },
    /// Instruction to change the value of a cell at a row/column reference of a
    /// specific sheet within a workpad.
    SetSheetCellValue {
        sheet_id: SheetId,
        row: usize,
        column: usize,
        value: String,
    },
}

/// Type to record the event history of a workpad
#[allow(dead_code)] // TODO Persistence
struct HistoryEntry {
    prior_version: Option<Version>,
    update: WorkpadUpdate,
}

/// Data that backs a [`WorkpadMaster`]
struct WorkpadMasterData {
    #[allow(dead_code)] // TODO Persistence
    id: String,
    history: RwLock<Vec<HistoryEntry>>,
    active_version: RwLock<Version>,
    next_part_id: AtomicU32,
    workpad_idx: VersionIndex<(), WorkpadData>,
    sheets_idx: VersionIndex<SheetId, SheetData>,
}

const NO_VER: &str = "Version not found";
impl WorkpadMasterData {
    /// Return the currently active version
    fn active_version(&self) -> Version {
        *self.active_version.read().unwrap()
    }

    /// Read workpad data for a specified version
    fn read_workpad(&self, version: Version) -> Arc<WorkpadData> {
        self.workpad_idx.read((), version).expect(NO_VER)
    }

    /// Write workpad data for a specified version
    fn write_workpad(&self, data: Arc<WorkpadData>, version: Version) {
        self.workpad_idx.write((), data, version);
    }

    /// Read sheet data for a specified version
    fn read_sheet(&self, id: SheetId, version: Version) -> Arc<SheetData> {
        self.sheets_idx.read(id, version).expect(NO_VER)
    }

    /// Write sheet data for a specified version
    fn write_sheet(&self, id: SheetId, data: Arc<SheetData>, version: Version) {
        self.sheets_idx.write(id, data, version);
    }

    /// Create a new sheet and insert it into the master data
    fn create_sheet(&mut self, version: Version, name: &str) -> SheetId {
        let id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();
        let columns = (0..99).map(ColumnData::new).collect();
        let rows = (0..999).map(RowData::new).collect();
        let data = SheetData {
            name: name.to_compact_string(),
            column_header_height: 20.0,
            row_header_width: 60.0,
            columns,
            rows,
            cells: HashMap::new(),
        };
        self.write_sheet(id, Arc::new(data), version);
        id
    }
}

/// Data structure to store information related only to the workpad.
#[derive(Debug, Clone)]
struct WorkpadData {
    name: CompactString,
    #[allow(dead_code)]
    sheets: Vec<SheetId>,
    active_sheet: SheetId,
}

/// A version of a workpad.  See [`WorkpadMaster::active_version()`].
pub struct Workpad {
    master: Arc<WorkpadMasterData>,
    version: Version,
    data: Arc<WorkpadData>,
}

impl Workpad {
    /// Create a new [`Workpad`] for a specific version
    fn new(master: Arc<WorkpadMasterData>, version: Version, data: Arc<WorkpadData>) -> Self {
        Self {
            master,
            version,
            data,
        }
    }

    /// Returns a [`Sheet`] representing the active sheet in this [`Workpad`]
    pub fn active_sheet(&self) -> Sheet {
        Sheet {
            id: self.data.active_sheet,
            data: self.master.read_sheet(self.data.active_sheet, self.version),
        }
    }

    /// Returns the name of the workpad
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Generate a [`WorkpadUpdate`] representing a change of name for this workpad.
    pub fn set_name(&mut self, new_name: String) -> WorkpadUpdate {
        WorkpadUpdate::SetWorkpadName { new_name }
    }

    /// Generate a [`WorkpadUpdate`] representing a change of name for the active
    /// sheet of this workpad.
    pub fn set_active_sheet_name(&mut self, new_name: String) -> WorkpadUpdate {
        WorkpadUpdate::SetSheetName {
            sheet_id: self.data.active_sheet,
            new_name,
        }
    }

    /// Generate a [`WorkpadUpdate`] representing a change of value for a cell in
    /// the active sheet of this workpad.
    pub fn set_active_sheet_cell_value(
        &mut self,
        row: usize,
        column: usize,
        value: String,
    ) -> WorkpadUpdate {
        WorkpadUpdate::SetSheetCellValue {
            sheet_id: self.data.active_sheet,
            row,
            column,
            value,
        }
    }
}

/// The Id for a sheet in a workpad
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SheetId(u32);

impl From<u32> for SheetId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for SheetId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SheetId({})", self.0)
    }
}

/// Data structure to store information related to a sheet within a workpad.
#[derive(Debug, Clone)]
pub struct SheetData {
    name: CompactString,
    column_header_height: f32,
    row_header_width: f32,
    columns: Vec<ColumnData>,
    rows: Vec<RowData>,
    // TODO need something that allows insertions/deletion of rows/columns
    cells: HashMap<(usize, usize), CellData>,
}

/// A sheet within a specific version of [`Workpad`].
pub struct Sheet {
    id: SheetId,
    data: Arc<SheetData>,
}

impl Sheet {
    #[allow(dead_code)]
    /// Returns the Id of the sheet.  The id remains constant across all versions of the workpad.
    pub fn id(&self) -> SheetId {
        self.id
    }

    /// Return the name of the sheet
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Return an iterator to the [`Column`]s held by this [`Sheet`]
    pub fn columns(&self) -> impl ExactSizeIterator<Item = Column<'_>> {
        self.data
            .columns
            .iter()
            .enumerate()
            .map(|(index, data)| Column::new(data, index))
    }

    /// Return a [`Column`] held by this [`Sheet`] given its index
    #[allow(dead_code)]
    pub fn column(&self, index: usize) -> Column<'_> {
        Column::new(&self.data.columns[index], index)
    }

    /// Return an iterator to the [`Rows`]s held by this [`Sheet`]
    pub fn rows(&self) -> impl ExactSizeIterator<Item = Row<'_>> {
        self.data
            .rows
            .iter()
            .enumerate()
            .map(|(index, data)| Row::new(data, index))
    }

    /// Return a [`Row`] held by this [`Sheet`] given its index
    #[allow(dead_code)]
    pub fn row(&self, index: usize) -> Row<'_> {
        Row::new(&self.data.rows[index], index)
    }

    /// Returns the height to be used for column headings when this
    /// [`Sheet`] is displayed.
    pub fn column_header_height(&self) -> f32 {
        self.data.column_header_height
    }

    /// Returns the width to be used for row headings when this
    /// [`Sheet`] is displayed.
    pub fn row_header_width(&self) -> f32 {
        self.data.row_header_width
    }

    #[allow(dead_code)]
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

    /// Return a [`Row`] held by this [`Sheet`] given its row and column indices.
    pub fn cell(&self, row: usize, column: usize) -> Cell<'_> {
        let cell_data = self
            .data
            .cells
            .get(&(row, column))
            .cloned()
            .unwrap_or_else(CellData::new);
        Cell::new(
            &self.data.rows[row],
            row,
            &self.data.columns[column],
            column,
            cell_data,
        )
    }
}

/// Data structure to store information related to a column of a sheet within a workpad.
#[derive(Debug, Clone)]
struct ColumnData {
    name: Name,
    width: f32,
}

impl ColumnData {
    // Create a new column data
    fn new(index: usize) -> Self {
        Self {
            name: Name::Auto(create_column_name(index)),
            width: 100.0,
        }
    }
}

/// A column within a specific version of a [`Sheet`] in a [`Workpad`].
#[allow(dead_code)]
pub struct Column<'pad> {
    data: &'pad ColumnData,
    index: usize,
}

#[allow(dead_code)]
impl Column<'_> {
    /// Create a new [`Column`]
    fn new(data: &'_ ColumnData, index: usize) -> Column<'_> {
        Column { data, index }
    }

    /// Return the index of this [`Column`]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the name of this [`Column`]
    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }

    /// Return the width of this [`Column`]
    pub fn width(&self) -> f32 {
        self.data.width
    }
}

/// Data structure to store information related to a row of a sheet within a workpad.
#[derive(Debug, Clone)]
struct RowData {
    name: Name,
    height: f32,
}

impl RowData {
    // Create a new row data
    fn new(index: usize) -> Self {
        Self {
            name: Name::Auto((index + 1).to_compact_string()),
            height: 20.0,
        }
    }
}

/// A row within a specific version of a [`Sheet`] in a [`Workpad`].
#[allow(dead_code)]
pub struct Row<'pad> {
    data: &'pad RowData,
    index: usize,
}

#[allow(dead_code)]
impl Row<'_> {
    /// Create a new [`Row`]
    fn new(data: &'_ RowData, index: usize) -> Row<'_> {
        Row { data, index }
    }

    /// Return the index of this [`Row`] within its [`Sheet`].
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the name of this [`Row`].
    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }

    /// Return the height of this [`Row`].
    pub fn height(&self) -> f32 {
        self.data.height
    }
}

/// Data structure to store information related to a cell within a workpad.
// TODO value types, borders, alignment, etc.
#[derive(Debug, Default, Clone)]
struct CellData {
    value: Value,
}

impl CellData {
    // Create a new cell data
    fn new() -> Self {
        Default::default()
    }
}

/// A cell within a specific version of a [`Workpad`].
pub struct Cell<'pad> {
    row_data: &'pad RowData,
    row_index: usize,
    column_data: &'pad ColumnData,
    column_index: usize,
    cell_data: CellData,
    name: Name,
}

#[allow(dead_code)]
impl Cell<'_> {
    /// Create a new [`Cell`]
    fn new<'pad>(
        row_data: &'pad RowData,
        row_index: usize,
        column_data: &'pad ColumnData,
        column_index: usize,
        cell_data: CellData,
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
            cell_data,
            name,
        }
    }

    /// Returns the [`Row`] to which this cell belongs
    // TODO Optional?
    pub fn row(&self) -> Row<'_> {
        Row::new(self.row_data, self.row_index)
    }

    /// Returns the [`Column`] to which this cell belongs
    // TODO Optional?
    pub fn column(&self) -> Column<'_> {
        Column::new(self.column_data, self.column_index)
    }

    /// Returns the width of this [`Cell`]
    // TODO Optional?
    pub fn width(&self) -> f32 {
        self.column_data.width
    }

    /// Returns the height of this [`Cell`]
    pub fn height(&self) -> f32 {
        self.row_data.height
    }

    /// Returns the name of this [`Cell`]
    pub fn name(&self) -> &str {
        match &self.name {
            Name::Auto(n) => n,
            Name::Custom(n) => n,
        }
    }

    /// Returns the value of this [`Cell`]
    // TODO How do we want to expose values?
    pub fn value(&self) -> &str {
        match &self.cell_data.value {
            Value::Empty => "",
            Value::String(s) => s,
        }
    }
}

/// A value of a cell
#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Empty,
    String(CompactString),
}

/// A name
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Name {
    /// The name is automatically derived
    Auto(CompactString),
    /// The name is explicitly set
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

struct VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord,
{
    index: RwLock<BTreeMap<(Id, Version, Version), Arc<Data>>>,
}

impl<Id, Data> VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord,
{
    fn read(&self, id: Id, version: Version) -> Option<Arc<Data>> {
        let index = self.index.read().unwrap();
        index
            .range(Self::all_versions(id))
            .find(|(&(_, from, to), _)| from <= version && version <= to)
            .map(|(_, v)| (*v).clone())
    }

    fn write(&self, id: Id, data: Arc<Data>, version: Version) {
        let mut index = self.index.write().unwrap();
        let existing = index
            .range(Self::all_versions(id))
            .find(|(&(_, from, to), _)| from <= version && version <= to)
            .map(|(k, v)| (k, (*v).clone()));
        match existing {
            Some((&(_, from, to), existing)) => {
                index.insert((id, from, version - 1), existing);
                index.insert((id, version, to), data);
                index.remove(&(id, from, to));
            }
            None => {
                index.insert((id, version, Version::MAX), data);
            }
        }
    }

    fn all_versions(id: Id) -> impl RangeBounds<(Id, Version, Version)> {
        struct AllVersions<Id> {
            start: (Id, Version, Version),
            end: (Id, Version, Version),
        }

        impl<Id> RangeBounds<(Id, Version, Version)> for AllVersions<Id> {
            fn start_bound(&self) -> std::ops::Bound<&(Id, Version, Version)> {
                std::ops::Bound::Included(&(self.start))
            }

            fn end_bound(&self) -> std::ops::Bound<&(Id, Version, Version)> {
                std::ops::Bound::Included(&(self.end))
            }
        }

        AllVersions {
            start: (id, 0, 0),
            end: (id, Version::MAX, Version::MAX),
        }
    }
}

impl<Id, Data> Default for VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord,
{
    fn default() -> Self {
        Self {
            index: Default::default(),
        }
    }
}
