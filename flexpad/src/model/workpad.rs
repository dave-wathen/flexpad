use std::{
    collections::BTreeMap,
    fmt,
    ops::RangeBounds,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
};

use internment::Intern;
use once_cell::sync::Lazy;
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

// TODO Update thread with message channel?
// TODO Feed versions to UI as required to trigger redraws/updates?
//

/// The version of a workpad
type Version = u32;

/// The type which underlies Id types
type IdBase = u32;
type IdBaseAtomic = AtomicU32;
/// Macro for defining an Id type
macro_rules! workpad_id_type {
    ($(
        #[$outer:meta])*
        $type_name:ident
    ) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $type_name(IdBase);

        impl From<IdBase> for $type_name {
            fn from(value: IdBase) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "$type_name({})", self.0)
            }
        }
    };
}

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
            next_part_id: Default::default(),
            workpad_idx: Default::default(),
            sheets_idx: Default::default(),
            columns_idx: Default::default(),
            rows_idx: Default::default(),
            cells_idx: Default::default(),
            sheets_cells_idx: Default::default(),
        };

        let sheet1_id = master_data.create_sheet(0, "Sheet 1");
        let sheet2_id = master_data.create_sheet(0, "Sheet 2");
        let sheet3_id = master_data.create_sheet(0, "Sheet 3");

        let workpad_data = WorkpadData {
            name: Intern::from("Unnamed"),
            author: Intern::from(whoami::realname().as_ref()),
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
            WorkpadUpdate::SetWorkpadProperties {
                new_name,
                new_author,
            } => {
                let workpad_data = self.data.read_workpad(active_version);
                let new_workpad_data = WorkpadData {
                    name: Intern::from(new_name.as_str()),
                    author: Intern::from(new_author.as_str()),
                    ..(*workpad_data).clone()
                };
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SetSheetName { sheet_id, new_name } => {
                let sheet_data = self.data.read_sheet(sheet_id, active_version);
                let new_sheet_data = SheetData {
                    name: Intern::from(new_name.as_str()),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(sheet_id, Arc::new(new_sheet_data), new_version);
            }
            WorkpadUpdate::SetSheetCellValue {
                sheet_id,
                row_id,
                column_id,
                value,
            } => {
                let active_cell_id =
                    self.data
                        .read_sheet_cell(sheet_id, row_id, column_id, active_version);
                let base = match active_cell_id {
                    Some(id) => (*self.data.read_cell(id, active_version)).clone(),
                    None => Default::default(),
                };
                let cell_data = CellData {
                    value: Value::String(Intern::from(value.as_str())),
                    ..base
                };

                // If there is already a CellId that covers the new version we can use it and
                // just write a new version for the CellData.  Otherwise allocate a new CellId.
                let cell_id = self
                    .data
                    .read_sheet_cell(sheet_id, row_id, column_id, new_version)
                    .unwrap_or_else(|| {
                        let new_id = self.data.next_part_id.fetch_add(1, Ordering::SeqCst).into();
                        self.data.write_sheet_cell(
                            sheet_id,
                            row_id,
                            column_id,
                            new_id,
                            new_version,
                        );
                        new_id
                    });
                self.data
                    .write_cell(cell_id, Arc::new(cell_data), new_version);
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
    SetWorkpadProperties {
        new_name: String,
        new_author: String,
    },
    /// Instruction to change the name of a specific sheet within a workpad.
    SetSheetName { sheet_id: SheetId, new_name: String },
    /// Instruction to change the value of a cell at a row/column reference of a
    /// specific sheet within a workpad.
    SetSheetCellValue {
        sheet_id: SheetId,
        row_id: RowId,
        column_id: ColumnId,
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
    next_part_id: IdBaseAtomic,
    workpad_idx: VersionIndex<(), Arc<WorkpadData>>,
    sheets_idx: VersionIndex<SheetId, Arc<SheetData>>,
    columns_idx: VersionIndex<ColumnId, Arc<ColumnData>>,
    rows_idx: VersionIndex<RowId, Arc<RowData>>,
    cells_idx: VersionIndex<CellId, Arc<CellData>>,
    sheets_cells_idx: VersionIndex<(SheetId, RowId, ColumnId), CellId>,
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
        let sheet_id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();

        let column_data = Arc::new(ColumnData {
            name: Name::Auto,
            width: 100.0,
        });
        let columns = (0..99)
            .map(|_| {
                let column_id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();
                self.write_column(column_id, column_data.clone(), version);
                column_id
            })
            .collect();

        let row_data = Arc::new(RowData {
            name: Name::Auto,
            height: 20.0,
        });
        let rows = (0..999)
            .map(|_| {
                let row_id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();
                self.write_row(row_id, row_data.clone(), version);
                row_id
            })
            .collect();

        let data = SheetData {
            name: Intern::from(name),
            column_header_height: 20.0,
            row_header_width: 60.0,
            columns,
            rows,
        };
        self.write_sheet(sheet_id, Arc::new(data), version);
        sheet_id
    }

    /// Read column data for a specified version
    fn read_column(&self, id: ColumnId, version: Version) -> Arc<ColumnData> {
        self.columns_idx.read(id, version).expect(NO_VER)
    }

    /// Write column data for a specified version
    fn write_column(&self, id: ColumnId, data: Arc<ColumnData>, version: Version) {
        self.columns_idx.write(id, data, version);
    }

    /// Read row data for a specified version
    fn read_row(&self, id: RowId, version: Version) -> Arc<RowData> {
        self.rows_idx.read(id, version).expect(NO_VER)
    }

    /// Write row data for a specified version
    fn write_row(&self, id: RowId, data: Arc<RowData>, version: Version) {
        self.rows_idx.write(id, data, version);
    }

    /// Read cell data for a specified version
    fn read_cell(&self, id: CellId, version: Version) -> Arc<CellData> {
        self.cells_idx.read(id, version).expect(NO_VER)
    }

    /// Write cell data for a specified version
    fn write_cell(&self, id: CellId, data: Arc<CellData>, version: Version) {
        self.cells_idx.write(id, data, version);
    }

    /// Read cell data for a specified version
    fn read_sheet_cell(
        &self,
        sheet_id: SheetId,
        row_id: RowId,
        column_id: ColumnId,
        version: Version,
    ) -> Option<CellId> {
        self.sheets_cells_idx
            .read((sheet_id, row_id, column_id), version)
    }

    /// Write cell data for a specified version
    fn write_sheet_cell(
        &self,
        sheet_id: SheetId,
        row_id: RowId,
        column_id: ColumnId,
        id: CellId,
        version: Version,
    ) {
        self.sheets_cells_idx
            .write((sheet_id, row_id, column_id), id, version);
    }
}

/// Data structure to store information related only to the workpad.
#[derive(Debug, Clone)]
struct WorkpadData {
    name: Intern<str>,
    author: Intern<str>,
    #[allow(dead_code)]
    sheets: Vec<SheetId>,
    active_sheet: SheetId,
}

/// A version of a workpad.  See [`WorkpadMaster::active_version()`].
#[derive(Clone)]
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
            master: self.master.clone(),
            version: self.version,
            id: self.data.active_sheet,
            data: self.master.read_sheet(self.data.active_sheet, self.version),
        }
    }

    /// Returns the name of the workpad
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Returns the name of the workpad
    pub fn author(&self) -> &str {
        &self.data.author
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
        self.active_sheet().set_cell_value(row, column, value)
    }
}

workpad_id_type!(
    #[doc = "The Id for a sheet in a workpad"]
    SheetId
);

/// Data structure to store information related to a sheet within a workpad.
#[derive(Debug, Clone)]
pub struct SheetData {
    name: Intern<str>,
    column_header_height: f32,
    row_header_width: f32,
    columns: Vec<ColumnId>,
    rows: Vec<RowId>,
}

/// A sheet within a specific version of [`Workpad`].
#[derive(Clone)]
pub struct Sheet {
    master: Arc<WorkpadMasterData>,
    version: Version,
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
    pub fn columns(&self) -> impl ExactSizeIterator<Item = Column> + '_ {
        (0..(self.data.columns.len())).map(|idx| self.column(idx))
    }

    /// Return a [`Column`] held by this [`Sheet`] given its index
    pub fn column(&self, index: usize) -> Column {
        let column_id = self.data.columns[index];
        let column_data = self.master.read_column(column_id, self.version);
        Column {
            data: column_data,
            index,
        }
    }

    /// Return an iterator to the [`Rows`]s held by this [`Sheet`]
    pub fn rows(&self) -> impl ExactSizeIterator<Item = Row> + '_ {
        (0..(self.data.rows.len())).map(|idx| self.row(idx))
    }

    /// Return a [`Row`] held by this [`Sheet`] given its index
    #[allow(dead_code)]
    pub fn row(&self, index: usize) -> Row {
        let row_id = self.data.rows[index];
        let row_data = self.master.read_row(row_id, self.version);
        Row {
            data: row_data,
            index,
        }
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
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
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
    pub fn cell(&self, row: usize, column: usize) -> Cell {
        let row_id = self.data.rows[row];
        let column_id = self.data.columns[column];
        let cell_id = self
            .master
            .read_sheet_cell(self.id, row_id, column_id, self.version);
        let cell_data = cell_id.map(|id| self.master.read_cell(id, self.version));
        Cell::new(self.row(row), self.column(column), cell_id, cell_data)
    }

    /// Generate a [`WorkpadUpdate`] representing a change of value for a cell in this sheet
    pub fn set_cell_value(&mut self, row: usize, column: usize, value: String) -> WorkpadUpdate {
        WorkpadUpdate::SetSheetCellValue {
            sheet_id: self.id,
            row_id: self.data.rows[row],
            column_id: self.data.columns[column],
            value,
        }
    }
}

workpad_id_type!(
    #[doc = "The Id for a column in a sheet of a workpad"]
    ColumnId
);

/// Data structure to store information related to a column of a sheet within a workpad.
#[derive(Debug, Clone)]
struct ColumnData {
    name: Name,
    width: f32,
}

/// A column within a specific version of a [`Sheet`] in a [`Workpad`].
#[derive(Debug, Clone)]
pub struct Column {
    data: Arc<ColumnData>,
    index: usize,
}

#[allow(dead_code)]
impl Column {
    /// Create a new [`Column`]
    fn new(data: Arc<ColumnData>, index: usize) -> Column {
        Column { data, index }
    }

    /// Return the index of this [`Column`]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the name of this [`Column`]
    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto => create_column_name(self.index()).as_ref(),
            Name::Custom(n) => n,
        }
    }

    /// Return the width of this [`Column`]
    pub fn width(&self) -> f32 {
        self.data.width
    }
}

workpad_id_type!(
    #[doc = "The Id for a row in a sheet of a workpad"]
    RowId
);

/// Data structure to store information related to a row of a sheet within a workpad.
#[derive(Debug, Clone)]
struct RowData {
    name: Name,
    height: f32,
}

/// A row within a specific version of a [`Sheet`] in a [`Workpad`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Row {
    data: Arc<RowData>,
    index: usize,
}

#[allow(dead_code)]
impl Row {
    /// Create a new [`Row`]
    fn new(data: Arc<RowData>, index: usize) -> Row {
        Row { data, index }
    }

    /// Return the index of this [`Row`] within its [`Sheet`].
    pub fn index(&self) -> usize {
        self.index
    }

    /// Return the name of this [`Row`].
    pub fn name(&self) -> &str {
        match &self.data.name {
            Name::Auto => intern_usize(self.index + 1).as_ref(),
            Name::Custom(n) => n,
        }
    }

    /// Return the height of this [`Row`].
    pub fn height(&self) -> f32 {
        self.data.height
    }
}

workpad_id_type!(
    #[doc = "The Id for a cell in a workpad"]
    CellId
);

/// Data structure to store information related to a cell within a workpad.
// TODO value types, borders, alignment, etc.
#[derive(Debug, Default, Clone)]
struct CellData {
    name: Name,
    value: Value,
}

/// A cell within a specific version of a [`Workpad`].
pub struct Cell {
    row: Row,
    column: Column,
    #[allow(dead_code)]
    id: Option<CellId>,
    data: Option<Arc<CellData>>,
}

#[allow(dead_code)]
impl Cell {
    /// Create a new [`Cell`]
    fn new(row: Row, column: Column, id: Option<CellId>, data: Option<Arc<CellData>>) -> Cell {
        Cell {
            row,
            column,
            id,
            data,
        }
    }

    /// Returns the width of this [`Cell`]
    // TODO Optional?
    pub fn width(&self) -> f32 {
        self.column.width()
    }

    /// Returns the height of this [`Cell`]
    pub fn height(&self) -> f32 {
        self.row.height()
    }

    /// Returns the name of this [`Cell`]
    pub fn name(&self) -> &str {
        let custom = match &self.data {
            Some(data) => match &data.name {
                Name::Auto => None,
                Name::Custom(n) => Some(n),
            },
            None => None,
        };
        match custom {
            Some(n) => n,
            None => Intern::new(format!("{}{}", self.column.name(), self.row.name())).as_ref(),
        }
    }

    /// Returns the value of this [`Cell`]
    // TODO How do we want to expose values?
    pub fn value(&self) -> &str {
        match &self.data {
            Some(data) => match &data.value {
                Value::Empty => "",
                Value::String(s) => s,
            },
            None => "",
        }
    }
}

/// A value of a cell
#[derive(Debug, Default, Clone)]
pub enum Value {
    #[default]
    Empty,
    String(Intern<str>),
}

/// A name
#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
enum Name {
    /// The name is automatically derived
    #[default]
    Auto,
    /// The name is explicitly set
    Custom(Intern<str>),
}

fn create_column_name(column: usize) -> Intern<str> {
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
        LO_1..=HI_1 => intern_base_26(column, 1),
        LO_2..=HI_2 => intern_base_26(column - LO_2, 2),
        LO_3..=HI_3 => intern_base_26(column - LO_3, 3),
        LO_4..=HI_4 => intern_base_26(column - LO_4, 4),
        LO_5..=HI_5 => intern_base_26(column - LO_5, 5),
        _ => panic!("Column too large"),
    }
}

fn intern_base_26(i: usize, min_len: usize) -> Intern<str> {
    let mut buffer = [b'A'; 20];
    let mut rem = i;
    let mut idx = 19;
    loop {
        buffer[idx] = b'A' + (rem % 26) as u8;
        if rem < 26 {
            break;
        }
        rem /= 26;
        idx -= 1;
    }
    Intern::from(unsafe { std::str::from_utf8_unchecked(&buffer[(20 - min_len)..20]) })
}

static LOW_NUMBERS: Lazy<[Intern<str>; 1000]> = Lazy::new(|| {
    let zero = Intern::from("0");
    let mut result: [Intern<str>; 1000] = [zero; 1000];
    (1..=999).for_each(|i| result[i] = intern_base_10(i));
    result
});

fn intern_usize(i: usize) -> Intern<str> {
    match i {
        0..=999 => LOW_NUMBERS[i],
        _ => intern_base_10(i),
    }
}

fn intern_base_10(i: usize) -> Intern<str> {
    let mut buffer = [b'0'; 20];
    let mut rem = i;
    let mut idx = 19;
    loop {
        buffer[idx] = b'0' + (rem % 10) as u8;
        if rem < 10 {
            break;
        }
        rem /= 10;
        idx -= 1;
    }
    Intern::from(unsafe { std::str::from_utf8_unchecked(&buffer[idx..20]) })
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
    Data: Clone,
{
    index: RwLock<BTreeMap<(Id, Version, Version), Data>>,
}

impl<Id, Data> VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord,
    Data: Clone,
{
    fn read(&self, id: Id, version: Version) -> Option<Data> {
        let index = self.index.read().unwrap();
        index
            .range(Self::all_versions(id))
            .find(|(&(_, from, to), _)| from <= version && version <= to)
            .map(|(_, v)| (*v).clone())
    }

    fn write(&self, id: Id, data: Data, version: Version) {
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
    Data: Clone,
{
    fn default() -> Self {
        Self {
            index: Default::default(),
        }
    }
}
