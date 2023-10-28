use std::{
    collections::BTreeMap,
    error::Error,
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

use crate::display_iter;

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
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $type_name(IdBase);

        impl From<IdBase> for $type_name {
            fn from(value: IdBase) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Debug for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", stringify!($type_name), self.0)
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", stringify!($type_name), self.0)
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
#[derive(Debug, Clone)]
pub struct WorkpadMaster {
    data: Arc<WorkpadMasterData>,
}

impl WorkpadMaster {
    /// Create a new [`WorkpadMaster`] representing a new workpad with a single
    // initial version.
    pub fn new() -> Self {
        Self::internal_new(false)
    }

    /// Create a new [`WorkpadMaster`] representing a new workpad with a single
    // initial version and three worksheets.
    pub fn new_starter() -> Self {
        Self::internal_new(true)
    }

    fn internal_new(starter: bool) -> Self {
        let master_data = WorkpadMasterData {
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

        let (sheets, active_sheet) = if starter {
            let sheet1_id = master_data.create_sheet(0, SheetKind::Worksheet, "Sheet 1");
            let sheet2_id = master_data.create_sheet(0, SheetKind::Worksheet, "Sheet 2");
            let sheet3_id = master_data.create_sheet(0, SheetKind::Worksheet, "Sheet 3");
            (vec![sheet1_id, sheet2_id, sheet3_id], Some(sheet1_id))
        } else {
            (vec![], None)
        };

        let workpad_data = WorkpadData {
            name: Intern::from("Unnamed"),
            author: Intern::from(whoami::realname().as_ref()),
            sheets,
            active_sheet,
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
        let version = self.data.active_version();
        let data = self.data.read_workpad(version);
        Workpad {
            master: self.clone(),
            version,
            data,
        }
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

        self.apply_update(update, active_version, new_version);
        *self.data.active_version.write().unwrap() = new_version;
    }

    pub fn apply_update(
        &mut self,
        update: WorkpadUpdate,
        active_version: Version,
        new_version: Version,
    ) {
        match update {
            WorkpadUpdate::Multi(updates) => {
                for update in updates {
                    self.apply_update(update, active_version, new_version)
                }
            }
            WorkpadUpdate::NewWorkpad => panic!("NewWorkpad not allowed for exisiting Workpad"),
            WorkpadUpdate::WorkpadSetProperties {
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
            WorkpadUpdate::SheetAdd { kind, name } => {
                let workpad_data = self.data.read_workpad(active_version);
                let sheet_id = self.data.create_sheet(new_version, kind, &name);
                let mut new_sheets = workpad_data.sheets.clone();
                new_sheets.push(sheet_id);
                let new_workpad_data = WorkpadData {
                    sheets: new_sheets,
                    active_sheet: Some(sheet_id),
                    ..(*workpad_data).clone()
                };
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SheetDelete { sheet_id } => {
                let workpad_data = self.data.read_workpad(active_version);
                let new_sheets: Vec<SheetId> = workpad_data
                    .sheets
                    .iter()
                    .copied()
                    .filter(|id| *id != sheet_id)
                    .collect();
                let new_active_sheet = if workpad_data.active_sheet == Some(sheet_id) {
                    let index = workpad_data
                        .sheets
                        .iter()
                        .position(|id| *id == sheet_id)
                        .unwrap();
                    if new_sheets.is_empty() {
                        None
                    } else if index < new_sheets.len() {
                        Some(new_sheets[index])
                    } else {
                        Some(new_sheets[0])
                    }
                } else {
                    workpad_data.active_sheet
                };
                let new_workpad_data = WorkpadData {
                    sheets: new_sheets,
                    active_sheet: new_active_sheet,
                    ..(*workpad_data).clone()
                };
                self.data.delete_sheet(sheet_id, new_version);
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SheetSetProperties { sheet_id, new_name } => {
                let sheet_data = self.data.read_sheet(sheet_id, active_version);
                let new_sheet_data = SheetData {
                    name: Intern::from(new_name.as_str()),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(sheet_id, Arc::new(new_sheet_data), new_version);
            }
            WorkpadUpdate::SheetSetCellValue {
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
            WorkpadUpdate::SheetSetActiveCell {
                sheet_id,
                row_id,
                column_id,
            } => {
                let sheet_data = self.data.read_sheet(sheet_id, active_version);
                let new_sheet_data = SheetData {
                    active_cell: Some((row_id, column_id)),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(sheet_id, Arc::new(new_sheet_data), new_version);
            }
        }
    }
}

impl Default for WorkpadMaster {
    fn default() -> Self {
        Self::new()
    }
}

/// A change that can be applied to a workpad to create a new version.
/// See [`WorkpadMaster::update(`)].
#[derive(Debug, Clone)]
pub enum WorkpadUpdate {
    /// Used to apply multiple updates in one version.
    Multi(Vec<WorkpadUpdate>),
    /// Used to represent the creation of a workpad.  See [`WorkpadMaster::new`].
    NewWorkpad,
    /// Instruction to change the name of the workpad
    WorkpadSetProperties {
        new_name: String,
        new_author: String,
    },
    /// Instruction to add a sheet to the workpad
    SheetAdd { kind: SheetKind, name: String },
    /// Instruction to delete a specific sheet within a workpad.
    SheetDelete { sheet_id: SheetId },
    /// Instruction to change the name of a specific sheet within a workpad.
    SheetSetProperties { sheet_id: SheetId, new_name: String },
    /// Instruction to change the value of a cell at a row/column reference of a
    /// specific sheet within a workpad.
    SheetSetCellValue {
        sheet_id: SheetId,
        row_id: RowId,
        column_id: ColumnId,
        value: String,
    },
    /// Instruction to change the active cell (row/column reference) of a
    /// specific sheet within a workpad.
    SheetSetActiveCell {
        sheet_id: SheetId,
        row_id: RowId,
        column_id: ColumnId,
    },
}

impl std::fmt::Display for WorkpadUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let WorkpadUpdate::Multi(updates) = self {
            let mut join = "";
            for update in updates {
                write!(f, "{}{}", join, update)?;
                join = " & ";
            }
            Ok(())
        } else {
            let name = match self {
                WorkpadUpdate::Multi(_) => unreachable!(),
                WorkpadUpdate::NewWorkpad => "New Workpad",
                WorkpadUpdate::WorkpadSetProperties { .. } => "Set Workpad Properties",
                WorkpadUpdate::SheetAdd { .. } => "Add Sheet",
                WorkpadUpdate::SheetDelete { .. } => "Delete Sheet",
                WorkpadUpdate::SheetSetProperties { .. } => "Set Sheet Properties",
                WorkpadUpdate::SheetSetCellValue { .. } => "Set Sheet Cell Value",
                WorkpadUpdate::SheetSetActiveCell { .. } => "Set Sheet Active Cell",
            };

            write!(f, "{name}")
        }
    }
}

// TODO Flesh out error
#[derive(Debug, Clone)]
pub struct UpdateError {}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "An Error occurred")
    }
}

impl Error for UpdateError {}

/// Type to record the event history of a workpad
#[allow(dead_code)] // TODO Persistence
#[derive(Debug)]
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

    /// Delete sheet data as of a specified version
    fn delete_sheet(&self, id: SheetId, version: Version) {
        self.sheets_idx.delete(id, version);
    }

    /// Create a new sheet and insert it into the master data
    fn create_sheet(&self, version: Version, kind: SheetKind, name: &str) -> SheetId {
        let sheet_id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();

        let column_data = Arc::new(ColumnData {
            name: Name::Auto,
            width: 100.0,
        });
        let columns: Vec<ColumnId> = (0..99)
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
        let rows: Vec<RowId> = (0..999)
            .map(|_| {
                let row_id = self.next_part_id.fetch_add(1, Ordering::SeqCst).into();
                self.write_row(row_id, row_data.clone(), version);
                row_id
            })
            .collect();

        let active_cell = Some((rows[0], columns[0]));

        let data = SheetData {
            kind,
            name: Intern::from(name),
            column_header_height: 20.0,
            row_header_width: 60.0,
            columns,
            rows,
            active_cell,
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

impl std::fmt::Debug for WorkpadMasterData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkpadMasterData")
            .field("id", &self.id)
            .field("history", &self.history)
            .field("active_version", &self.active_version)
            .finish()
    }
}

/// Data structure to store information related only to the workpad.
#[derive(Debug, Clone)]
struct WorkpadData {
    name: Intern<str>,
    author: Intern<str>,
    sheets: Vec<SheetId>,
    active_sheet: Option<SheetId>,
}

/// A version of a workpad.  See [`WorkpadMaster::active_version()`].
#[derive(Clone, Debug)]
pub struct Workpad {
    master: WorkpadMaster,
    version: Version,
    data: Arc<WorkpadData>,
}

impl Workpad {
    pub fn id(&self) -> &str {
        &self.master.data.id
    }

    // Returns the [`WorkpadMaster`] of this [`Workpad`]
    pub fn master(&self) -> WorkpadMaster {
        self.master.clone()
    }

    /// Returns a [`Sheet`] representing the active sheet in this [`Workpad`]
    pub fn sheets(&self) -> impl Iterator<Item = Sheet> + '_ {
        self.data
            .sheets
            .iter()
            .map(|id| self.sheet_by_id(*id).unwrap())
    }

    /// Returns a [`Sheet`] representing the active sheet in this [`Workpad`]
    pub fn active_sheet(&self) -> Option<Sheet> {
        self.data
            .active_sheet
            .map(|id| self.sheet_by_id(id).unwrap())
    }

    /// Returns a [`Sheet`] representing the active sheet in this [`Workpad`]
    pub fn sheet_by_id(&self, id: SheetId) -> Option<Sheet> {
        self.data.sheets.iter().any(|x| *x == id).then(|| Sheet {
            workpad: self.clone(),
            version: self.version,
            id,
            data: self.master.data.read_sheet(id, self.version),
        })
    }

    /// Returns the name of the workpad
    pub fn name(&self) -> &str {
        &self.data.name
    }

    /// Returns the name of the workpad
    pub fn author(&self) -> &str {
        &self.data.author
    }
}

workpad_id_type!(
    #[doc = "The Id for a sheet in a workpad"]
    SheetId
);

impl std::fmt::Display for Workpad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //        write!(f, "Workpad{{id: {}, version: {}}}", self.id(), self.version)
        f.write_str("Workpad{")?;
        {
            f.write_str("id:")?;
            self.id().fmt(f)?;
            f.write_str(", version:")?;
            self.version.fmt(f)?;
            f.write_str(", sheets:")?;
            display_iter(self.data.sheets.iter(), f)?;
            f.write_str(", active_sheet:")?;
            match &self.data.active_sheet {
                Some(id) => id.fmt(f)?,
                None => f.write_str("None")?,
            }
        }
        f.write_str("}")
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SheetKind {
    #[default]
    Worksheet,
    Textsheet,
}

impl std::fmt::Display for SheetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SheetKind::Worksheet => write!(f, "Worksheet"),
            SheetKind::Textsheet => write!(f, "Textsheet"),
        }
    }
}

/// Data structure to store information related to a sheet within a workpad.
#[derive(Debug, Clone)]
pub struct SheetData {
    #[allow(dead_code)] // TODO Only Worksheet supported so far
    kind: SheetKind,
    name: Intern<str>,
    column_header_height: f32,
    row_header_width: f32,
    columns: Vec<ColumnId>,
    rows: Vec<RowId>,
    // Active cell is (RowId, ColumnId) rather than CellId as the active cell may be empty
    active_cell: Option<(RowId, ColumnId)>,
}

/// A sheet within a specific version of [`Workpad`].
#[derive(Debug, Clone)]
pub struct Sheet {
    workpad: Workpad,
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

    /// Returns the [`Workpad`] of this [`Sheet`]
    pub fn workpad(&self) -> Workpad {
        self.workpad.clone()
    }

    /// Return the [`SheetKind`] of the sheet
    pub fn kind(&self) -> SheetKind {
        SheetKind::Worksheet
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
        let id = self.data.columns[index];
        let master_data = &self.workpad.master.data;
        let data = master_data.read_column(id, self.version);
        Column {
            sheet: self.clone(),
            data,
            id,
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
        let id = self.data.rows[index];
        let master_data = &self.workpad.master.data;
        let data = master_data.read_row(id, self.version);
        Row {
            sheet: self.clone(),
            data,
            id,
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

    /// Returns the row and column indices or the active cell of this ['Sheet'].
    /// If there are no cells in this sheet it will return `None`
    pub fn active_cell(&self) -> Option<(usize, usize)> {
        self.data.active_cell.map(|(row_id, column_id)| {
            let row_idx = self.data.rows.iter().position(|id| *id == row_id).unwrap();
            let column_idx = self
                .data
                .columns
                .iter()
                .position(|id| *id == column_id)
                .unwrap();
            (row_idx, column_idx)
        })
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
        let master_data = &self.workpad.master.data;
        let id = master_data.read_sheet_cell(self.id, row_id, column_id, self.version);
        let data = id.map(|id| master_data.read_cell(id, self.version));
        Cell {
            sheet: self.clone(),
            row: self.row(row),
            column: self.column(column),
            id,
            data,
        }
    }
}

impl std::fmt::Display for Sheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sheet{{")?;
        f.write_str("id:")?;
        self.id().fmt(f)?;
        f.write_str("workpad_id:")?;
        self.workpad.id().fmt(f)?;
        f.write_str(", version:")?;
        self.workpad.version.fmt(f)?;
        f.write_str(", rows:")?;
        display_iter(self.data.rows.iter(), f)?;
        f.write_str(", columns:")?;
        display_iter(self.data.columns.iter(), f)?;
        write!(f, "}}",)
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
    sheet: Sheet,
    data: Arc<ColumnData>,
    id: ColumnId,
    index: usize,
}

#[allow(dead_code)]
impl Column {
    /// Returns the Id of the column.  The id remains constant across all versions of the workpad.
    pub fn id(&self) -> ColumnId {
        self.id
    }

    // Returns the [`Sheet`] of this [`Column`]
    pub fn sheet(&self) -> Sheet {
        self.sheet.clone()
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
    sheet: Sheet,
    data: Arc<RowData>,
    id: RowId,
    index: usize,
}

#[allow(dead_code)]
impl Row {
    /// Returns the Id of the row.  The id remains constant across all versions of the workpad.
    pub fn id(&self) -> RowId {
        self.id
    }

    // Returns the [`Sheet`] of this [`Row`]
    pub fn sheet(&self) -> Sheet {
        self.sheet.clone()
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
#[derive(Debug)]
pub struct Cell {
    sheet: Sheet,
    row: Row,
    column: Column,
    #[allow(dead_code)]
    id: Option<CellId>,
    data: Option<Arc<CellData>>,
}

#[allow(dead_code)]
impl Cell {
    // Returns the [`Sheet`] of this [`Cell`]
    pub fn sheet(&self) -> Sheet {
        self.sheet.clone()
    }

    /// Returns the [`Row`] of this [`Cell`]
    pub fn row(&self) -> Row {
        self.row.clone()
    }

    /// Returns the [`Column`] of this [`Cell`]
    pub fn column(&self) -> Column {
        self.column.clone()
    }

    /// Returns the width of this [`Cell`]
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
    /// Reads data as at the given version:
    fn read(&self, id: Id, version: Version) -> Option<Data> {
        let index = self.index.read().unwrap();
        index
            .range(Self::all_versions(id))
            .find(|(&(_, from, to), _)| from <= version && version <= to)
            .map(|(_, v)| (*v).clone())
    }

    /// Writes new data as at the given version:
    ///
    /// # before
    /// * (from, max, Id) -> existing_data
    ///
    /// # after
    /// * (from, version-1, id) -> existing_data
    /// * (version, max id) -> new_data
    ///
    /// or if no existing_data, only the new data entry is made
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

    /// Deletes as at the given version:
    ///
    /// # before
    /// * (from, max, Id) -> data
    ///
    /// # after
    /// * (from, version-1, Id) -> data
    fn delete(&self, id: Id, version: Version) {
        let mut index = self.index.write().unwrap();
        let existing = index
            .range(Self::all_versions(id))
            .find(|(&(_, from, to), _)| from <= version && version <= to)
            .map(|(k, v)| (k, (*v).clone()));
        if let Some((&(_, from, to), existing)) = existing {
            index.insert((id, from, version - 1), existing);
            index.remove(&(id, from, to));
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
