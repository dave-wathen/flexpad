use std::{
    borrow::Borrow,
    collections::{btree_map::Range, BTreeMap},
    error::Error,
    fmt,
    ops::RangeBounds,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, RwLock,
    },
};

use internment::Intern;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rust_i18n::{i18n, t};
use uuid::Uuid;

//use crate::display_iter;

i18n!("locales", fallback = "en");

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

/// The [`Result`] type returned when a workpad is updated.  On Success the
/// Workpad represents the newly created version.
pub type UpdateResult = Result<Workpad, UpdateError>;

/// The version of a workpad
pub type Version = u32;

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
    pub fn new_blank() -> Self {
        Self::internal_new(false)
    }

    /// Create a new [`WorkpadMaster`] representing a new workpad with a single
    // initial version and three worksheets.
    pub fn new_starter() -> Self {
        Self::internal_new(true)
    }

    fn internal_new(starter: bool) -> Self {
        let update = WorkpadUpdate::NewWorkpad;

        let master_data = WorkpadMasterData {
            id: Uuid::new_v4().simple().to_string(),
            transaction: RwLock::new(None),
            history: Default::default(),
            active_version: RwLock::new(0),
            next_part_id: Default::default(),
            workpad_idx: Default::default(),
            sheets_idx: Default::default(),
            columns_idx: Default::default(),
            rows_idx: Default::default(),
            cells_idx: Default::default(),
            sheets_cells_idx: Default::default(),
        };

        let tx = master_data.tx_begin();

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
        master_data.tx_commit(&tx, update);

        WorkpadMaster {
            data: Arc::new(master_data),
        }
    }

    /// The id of this workpad
    pub fn id(&self) -> &str {
        &self.data.id
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
    pub fn update(&mut self, update: WorkpadUpdate) -> UpdateResult {
        if let WorkpadUpdate::SetVersion { version } = update {
            let history = self.data.history.read().unwrap();
            if (version as usize) < history.len() {
                self.data.set_version(version);
                Ok(self.active_version())
            } else {
                Err(UpdateError {
                    kind: ErrorKind::MissingVersion(version),
                    update: update.clone(),
                    workpad_id: self.data.id.clone(),
                    workpad_version: *self.data.active_version.read().unwrap(),
                })
            }
        } else {
            let tx = self.data.tx_begin();

            match self.apply_update(&update, &tx) {
                Ok(_) => {
                    self.data.tx_commit(&tx, update);
                    Ok(self.active_version())
                }
                Err(err) => {
                    self.data.tx_rollback(&tx);
                    Err(err)
                }
            }
        }
    }

    fn apply_update(
        &mut self,
        update: &WorkpadUpdate,
        tx: &Transaction,
    ) -> Result<(), UpdateError> {
        let new_err = |kind| {
            Err(UpdateError {
                kind,
                update: update.clone(),
                workpad_id: self.data.id.clone(),
                workpad_version: tx.active_version,
            })
        };

        let Transaction {
            id: _,
            active_version,
            new_version,
        } = *tx;

        match update {
            WorkpadUpdate::Multi(updates) => {
                for update in updates {
                    self.apply_update(update, tx)?;
                }
            }
            WorkpadUpdate::NewWorkpad => panic!("NewWorkpad not allowed for existing Workpad"),
            WorkpadUpdate::SetVersion { .. } => unreachable!(),
            WorkpadUpdate::WorkpadSetProperties {
                ref new_name,
                ref new_author,
            } => {
                if new_name.is_empty() {
                    return new_err(ErrorKind::InvalidName(new_name.clone()));
                }
                let workpad_data = self.data.read_workpad(active_version);
                let new_workpad_data = WorkpadData {
                    name: Intern::from(new_name.as_str()),
                    author: Intern::from(new_author.as_str()),
                    ..(*workpad_data).clone()
                };
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SetActiveSheet { sheet_id } => {
                let workpad_data = self.data.read_workpad(active_version);
                if !workpad_data.sheets.contains(sheet_id) {
                    return new_err(ErrorKind::MissingSheet(*sheet_id));
                }

                let new_workpad_data = WorkpadData {
                    active_sheet: Some(*sheet_id),
                    ..(*workpad_data).clone()
                };
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SheetAdd { kind, ref name } => {
                if name.is_empty() {
                    return new_err(ErrorKind::InvalidName(name.clone()));
                }

                let workpad_data = self.data.read_workpad(active_version);
                for sheet_id in workpad_data.sheets.iter() {
                    let sheet_data = self.data.read_sheet(*sheet_id, active_version);
                    let sheet_name: &str = &sheet_data.name;
                    if sheet_name == name {
                        return new_err(ErrorKind::DuplicateName(name.clone()));
                    }
                }

                let sheet_id = self.data.create_sheet(new_version, *kind, name);
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
                    .filter(|id| *id != *sheet_id)
                    .collect();

                if new_sheets.len() == workpad_data.sheets.len() {
                    return new_err(ErrorKind::MissingSheet(*sheet_id));
                }

                let new_active_sheet = if workpad_data.active_sheet == Some(*sheet_id) {
                    let index = workpad_data
                        .sheets
                        .iter()
                        .position(|id| id == sheet_id)
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
                self.data.delete_sheet(*sheet_id, new_version);
                self.data
                    .write_workpad(Arc::new(new_workpad_data), new_version);
            }
            WorkpadUpdate::SheetSetProperties {
                sheet_id,
                ref new_name,
            } => {
                if new_name.is_empty() {
                    return new_err(ErrorKind::InvalidName(new_name.clone()));
                }

                let workpad_data = self.data.read_workpad(active_version);
                for s_id in workpad_data.sheets.iter() {
                    let sheet_data = self.data.read_sheet(*s_id, active_version);
                    let sheet_name: &str = &sheet_data.name;
                    if s_id != sheet_id && sheet_name == new_name {
                        return new_err(ErrorKind::DuplicateName(new_name.clone()));
                    }
                }

                let sheet_data = self.data.read_sheet(*sheet_id, active_version);
                let new_sheet_data = SheetData {
                    name: Intern::from(new_name.as_str()),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(*sheet_id, Arc::new(new_sheet_data), new_version);
            }
            WorkpadUpdate::SheetSetCellValue {
                sheet_id,
                row_id,
                column_id,
                ref value,
            } => {
                let workpad_data = self.data.read_workpad(active_version);
                if !workpad_data.sheets.contains(sheet_id) {
                    return new_err(ErrorKind::MissingSheet(*sheet_id));
                }

                let sheet_data = self.data.read_sheet(*sheet_id, active_version);
                if !sheet_data.rows.contains(row_id) {
                    return new_err(ErrorKind::MissingRow(*row_id));
                }
                if !sheet_data.columns.contains(column_id) {
                    return new_err(ErrorKind::MissingColumn(*column_id));
                }

                let cell_id =
                    self.data
                        .read_sheet_cell(*sheet_id, *row_id, *column_id, active_version);
                let base = match cell_id {
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
                    .read_sheet_cell(*sheet_id, *row_id, *column_id, new_version)
                    .unwrap_or_else(|| {
                        let new_id = self.data.next_part_id.fetch_add(1, Ordering::SeqCst).into();
                        self.data.write_sheet_cell(
                            *sheet_id,
                            *row_id,
                            *column_id,
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
                let workpad_data = self.data.read_workpad(active_version);
                if !workpad_data.sheets.contains(sheet_id) {
                    return new_err(ErrorKind::MissingSheet(*sheet_id));
                }

                let sheet_data = self.data.read_sheet(*sheet_id, active_version);
                if !sheet_data.rows.contains(row_id) {
                    return new_err(ErrorKind::MissingRow(*row_id));
                }
                if !sheet_data.columns.contains(column_id) {
                    return new_err(ErrorKind::MissingColumn(*column_id));
                }

                let new_sheet_data = SheetData {
                    active_cell: Some((*row_id, *column_id)),
                    ..(*sheet_data).clone()
                };
                self.data
                    .write_sheet(*sheet_id, Arc::new(new_sheet_data), new_version);
            }
        }
        Ok(())
    }
}

impl Default for WorkpadMaster {
    fn default() -> Self {
        Self::new_blank()
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
    /// Instruction to change the active version of the workpad
    SetVersion { version: Version },
    /// Instruction to change the name of the workpad
    WorkpadSetProperties {
        new_name: String,
        new_author: String,
    },
    /// Instruction to change the active sheet of the workpad
    SetActiveSheet { sheet_id: SheetId },
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
            let mut join = String::new();
            for update in updates {
                write!(f, "{}{}", join, update)?;
                join = t!("WorkpadUpdate.Join");
            }
            Ok(())
        } else {
            type WU = WorkpadUpdate;
            let variant = match self {
                WU::Multi(_) => unreachable!(),
                WU::NewWorkpad => "NewWorkpad",
                WU::SetVersion { .. } => "SetVersion",
                WU::WorkpadSetProperties { .. } => "WorkpadSetProperties",
                WU::SetActiveSheet { .. } => "SetActiveSheet",
                WU::SheetAdd { .. } => "SheetAdd",
                WU::SheetDelete { .. } => "SheetDelete",
                WU::SheetSetProperties { .. } => "SheetSetProperties",
                WU::SheetSetCellValue { .. } => "SheetSetCellValue",
                WU::SheetSetActiveCell { .. } => "SheetSetActiveCell",
            };
            let name = t!(&format!("WorkpadUpdate.{variant}"));
            write!(f, "{name}")
        }
    }
}

// TODO Flesh out error
#[derive(Debug, Clone)]
pub struct UpdateError {
    kind: ErrorKind,
    update: WorkpadUpdate,
    workpad_id: String,
    workpad_version: Version,
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &t!("UpdateError.Display")
                .replace("{kind}", &self.kind.to_string())
                .replace("{update}", &self.update.to_string())
                .replace("{workpad_id}", &self.workpad_id)
                .replace("{workpad_versiojn}", &self.workpad_version.to_string()),
        )
    }
}

impl Error for UpdateError {}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidName(String),
    MissingVersion(Version),
    MissingSheet(SheetId),
    MissingRow(RowId),
    MissingColumn(ColumnId),
    DuplicateName(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => {
                f.write_str(&t!("UpdateError.InvalidName").replace("{name}", name))
            }
            Self::MissingVersion(version) => f.write_str(
                &t!("UpdateError.MissingVersion").replace("{version}", &version.to_string()),
            ),
            Self::MissingSheet(id) => {
                f.write_str(&t!("UpdateError.MissingId").replace("{id}", &id.to_string()))
            }
            Self::MissingRow(id) => {
                f.write_str(&t!("UpdateError.MissingId").replace("{id}", &id.to_string()))
            }
            Self::MissingColumn(id) => {
                f.write_str(&t!("UpdateError.MissingId").replace("{id}", &id.to_string()))
            }
            Self::DuplicateName(name) => {
                f.write_str(&t!("UpdateError.DuplicateName").replace("{name}", name))
            }
        }
    }
}

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
    transaction: RwLock<Option<Transaction>>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Transaction {
    id: Version,
    active_version: Version,
    new_version: Version,
}

const NO_VER: &str = "Version not found";
impl WorkpadMasterData {
    /// Start a transaction.  Transactions cannot be concurrent so no existing transaction should exist.
    fn tx_begin(&self) -> Transaction {
        let mut tx = self.transaction.write().unwrap();
        assert!(tx.is_none(), "Concurrent transactions not supported");

        let new_tx = Transaction {
            id: self.next_part_id.fetch_add(1, Ordering::SeqCst),
            active_version: self.active_version(),
            new_version: self.history.read().unwrap().len() as Version,
        };
        tx.replace(new_tx);

        self.workpad_idx.tx_begin();
        self.sheets_idx.tx_begin();
        self.columns_idx.tx_begin();
        self.rows_idx.tx_begin();
        self.cells_idx.tx_begin();
        self.sheets_cells_idx.tx_begin();

        new_tx
    }

    /// Commit the current transaction.
    fn tx_commit(&self, tx: &Transaction, update: WorkpadUpdate) {
        let mut self_tx = self.transaction.write().unwrap();
        match self_tx.as_ref() {
            Some(inner) => assert!(inner == tx, "Transaction is not in progress"),
            None => panic!("No transaction is in progress"),
        }

        self.workpad_idx.tx_commit();
        self.sheets_idx.tx_commit();
        self.columns_idx.tx_commit();
        self.rows_idx.tx_commit();
        self.cells_idx.tx_commit();
        self.sheets_cells_idx.tx_commit();

        let mut history = self.history.write().unwrap();
        let prior_version = if let WorkpadUpdate::NewWorkpad = update {
            None
        } else {
            Some(tx.active_version)
        };

        history.push(HistoryEntry {
            prior_version,
            update,
        });
        self.set_version(tx.new_version);
        self_tx.take();
    }

    /// Commit the current transaction.
    fn tx_rollback(&self, tx: &Transaction) {
        let mut self_tx = self.transaction.write().unwrap();
        match self_tx.as_ref() {
            Some(inner) => assert!(inner == tx, "Transaction is not in progress"),
            None => panic!("No transaction is in progress"),
        }

        self.next_part_id.store(tx.id, Ordering::SeqCst);
        self.workpad_idx.tx_rollback();
        self.sheets_idx.tx_rollback();
        self.columns_idx.tx_rollback();
        self.rows_idx.tx_rollback();
        self.cells_idx.tx_rollback();
        self.sheets_cells_idx.tx_rollback();

        self_tx.take();
    }

    fn set_version(&self, new_version: Version) {
        *self.active_version.write().unwrap() = new_version;
    }

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

    /// Returns the [`WorkpadMaster`] of this [`Workpad`]
    pub fn master(&self) -> WorkpadMaster {
        self.master.clone()
    }

    /// Returns the id of this version and a description of the update that created it.
    pub fn version(&self) -> (Version, String) {
        let history = self.master.data.history.read().unwrap();
        (
            self.version,
            history[self.version as usize].update.to_string(),
        )
    }

    /// Returns the version information (see [`Workpad::version`]) for the versions which
    /// preceed this version.  Versions are returned from the immediate predecessor of
    /// this version backwards.
    #[allow(dead_code)]
    pub fn backward_versions(&self) -> impl Iterator<Item = (Version, String)> {
        let history = self.master.data.history.read().unwrap();
        let mut entry = &history[self.version as usize];

        let mut versions = vec![];
        while let Some(prior_version) = entry.prior_version {
            entry = &history[prior_version as usize];
            versions.push((prior_version, entry.update.to_string()))
        }

        versions.into_iter()
    }

    /// Returns the version information (see [`Workpad::version`]) for the versions which
    /// succeed this version.  Versions are returned from the the immediate successor of
    /// this version forwards.
    #[allow(dead_code)]
    pub fn forward_versions(&self) -> impl Iterator<Item = (Version, String)> {
        let history = self.master.data.history.read().unwrap();
        // history cannot be empty otherwise this version could not exist
        let mut ver = (history.len() - 1) as Version;

        let mut versions = vec![];
        while ver != self.version {
            let entry = &history[ver as usize];
            versions.push((ver as Version, entry.update.to_string()));
            // Prior version must exist until we reach this version
            ver = entry.prior_version.unwrap();
        }

        versions.into_iter().rev()
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
        f.write_str("Workpad{")?;
        {
            f.write_str("id:")?;
            self.id().fmt(f)?;
            f.write_str(", version:")?;
            self.version.fmt(f)?;
            f.write_str(", sheets:[")?;
            f.write_fmt(format_args!("{}", self.data.sheets.iter().format(", ")))?;
            f.write_str("], active_sheet:")?;
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
    /// Returns the Id of the sheet.  The id remains constant across all versions of the workpad.
    pub fn id(&self) -> SheetId {
        self.id
    }

    /// Returns the [`Workpad`] of this [`Sheet`]
    pub fn workpad(&self) -> Workpad {
        self.workpad.clone()
    }

    /// Return the [`SheetKind`] of the sheet
    #[allow(dead_code)]
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

    /// Returns the active [`Cell`] of this ['Sheet'].
    /// If there are no cells in this sheet it will return `None`
    pub fn active_cell(&self) -> Option<Cell> {
        self.data.active_cell.map(|(row_id, column_id)| {
            self.internal_cell(
                self.internal_row_index(row_id),
                row_id,
                self.internal_column_index(column_id),
                column_id,
            )
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
        self.internal_cell(row, row_id, column, column_id)
    }

    pub fn internal_cell(
        &self,
        row: usize,
        row_id: RowId,
        column: usize,
        column_id: ColumnId,
    ) -> Cell {
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

    fn internal_row_index(&self, row_id: RowId) -> usize {
        self.data.rows.iter().position(|id| *id == row_id).unwrap()
    }

    fn internal_column_index(&self, column_id: ColumnId) -> usize {
        self.data
            .columns
            .iter()
            .position(|id| *id == column_id)
            .unwrap()
    }
}

impl PartialEq for Sheet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.version == other.version
    }
}

impl Eq for Sheet {}

impl std::fmt::Display for Sheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sheet{{")?;
        f.write_str("id:")?;
        self.id().fmt(f)?;
        f.write_str("workpad_id:")?;
        self.workpad.id().fmt(f)?;
        f.write_str(", version:")?;
        self.workpad.version.fmt(f)?;
        f.write_str(", rows:[")?;
        f.write_fmt(format_args!("{}", self.data.rows.iter().format(", ")))?;
        f.write_str("], columns:[")?;
        f.write_fmt(format_args!("{}", self.data.columns.iter().format(", ")))?;
        write!(f, "]}}",)
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

#[derive(Debug, Clone)]
enum UndoAction<Key, Data> {
    Insert(Key, Data),
    Delete(Key),
}

#[derive(Debug)]
struct UndoableIndex<Key, Value>
where
    Key: Copy + std::cmp::Ord,
    Value: Clone,
{
    index: BTreeMap<Key, Value>,
    undos: Vec<UndoAction<Key, Value>>,
}

impl<Key, Value> UndoableIndex<Key, Value>
where
    Key: Copy + std::cmp::Ord,
    Value: Clone,
{
    fn tx_begin(&self) {
        assert!(self.undos.is_empty(), "Previous transaction incomplete");
    }

    fn tx_commit(&mut self) {
        self.undos.clear();
    }

    fn tx_rollback(&mut self) {
        let undos = std::mem::take(&mut self.undos);
        for undo in undos {
            match undo {
                UndoAction::Insert(key, value) => self.index.insert(key, value),
                UndoAction::Delete(key) => self.index.remove(&key),
            };
        }
    }

    fn range<T, R>(&self, range: R) -> Range<'_, Key, Value>
    where
        T: ?Sized + std::cmp::Ord,
        Key: Borrow<T> + std::cmp::Ord,
        R: RangeBounds<T>,
    {
        self.index.range(range)
    }

    fn insert(&mut self, key: Key, value: Value) {
        match self.index.insert(key, value) {
            Some(prior) => self.undos.push(UndoAction::Insert(key, prior)),
            None => self.undos.push(UndoAction::Delete(key)),
        };
    }

    fn remove(&mut self, key: &Key) {
        if let Some(prior) = self.index.remove(key) {
            self.undos.push(UndoAction::Insert(*key, prior))
        };
    }
}

impl<Key, Value> Default for UndoableIndex<Key, Value>
where
    Key: Copy + Ord,
    Value: Clone,
{
    fn default() -> Self {
        Self {
            index: Default::default(),
            undos: Default::default(),
        }
    }
}

#[derive(Debug)]
struct VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord,
    Data: Clone,
{
    index: RwLock<UndoableIndex<(Id, Version, Version), Data>>,
}

impl<Id, Data> VersionIndex<Id, Data>
where
    Id: Copy + std::cmp::Ord + std::fmt::Debug,
    Data: Clone + std::fmt::Debug,
{
    fn tx_begin(&self) {
        self.index.read().unwrap().tx_begin();
    }

    fn tx_commit(&self) {
        self.index.write().unwrap().tx_commit();
    }

    fn tx_rollback(&self) {
        self.index.write().unwrap().tx_rollback();
    }

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
            Some((&(_, from, to), existing)) if from < version => {
                index.insert((id, from, version - 1), existing);
                index.insert((id, version, to), data);
                index.remove(&(id, from, to));
            }
            Some((&(_, from, to), _)) => {
                index.insert((id, from, to), data);
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

    #[test]
    fn new_blank() {
        let master = WorkpadMaster::new_blank();
        let pad = master.active_version();

        // Assert no sheets and no active sheet
        assert!(pad.active_sheet().is_none());
        assert!(pad.sheets().next().is_none());

        // Assert now at version 0 created by "New Workpad""
        assert!(ver_is(pad.version(), 0, "New Workpad"));

        // Assert no backward_versions or forward_versions
        assert!(pad.backward_versions().next().is_none());
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn new_starter() {
        let master = WorkpadMaster::new_starter();
        let pad = master.active_version();

        // Assert sheets is: ["Sheet 1", "Sheet 2", "Sheet 3"] and "Sheet 1" is the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "Sheet 1", &pad);
        assert_next_sheet(&mut sheets, "Sheet 2");
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());

        // Assert now at version 0 created by "New Workpad""
        assert!(ver_is(pad.version(), 0, "New Workpad"));

        // Assert no backward_versions or forward_versions
        assert!(pad.backward_versions().next().is_none());
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn set_workpad_properties() {
        let mut master = WorkpadMaster::new_blank();

        // Set pad properties
        let pad = master
            .update(WorkpadUpdate::WorkpadSetProperties {
                new_name: String::from("Test Workpad"),
                new_author: String::from("A Writer"),
            })
            .expect("Update should succeed");

        assert_eq!("Test Workpad", pad.name());
        assert_eq!("A Writer", pad.author());

        // Assert now at version 1 created by "Set Workpad Properties""
        assert!(ver_is(pad.version(), 1, "Set Workpad Properties"));

        // Assert backward_versions is [(0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn cannot_set_workpad_properties_with_blank_name() {
        let mut master = WorkpadMaster::new_blank();

        // Set pad properties
        let result = master.update(WorkpadUpdate::WorkpadSetProperties {
            new_name: String::from(""),
            new_author: String::from("A Writer"),
        });

        assert!(result.is_err());
        assert_eq!(
            r#"The name "" is not allowed (during update: Set Workpad Properties)"#,
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn set_workpad_active_sheet() {
        let mut master = WorkpadMaster::new_starter();

        let pad = master.active_version();
        let sheet_2_id = pad.sheets().nth(1).unwrap().id();

        // Change active sheet
        let pad = master
            .update(WorkpadUpdate::SetActiveSheet {
                sheet_id: sheet_2_id,
            })
            .expect("Update should succeed");

        // Assert sheets is: ["Sheet 1", "Sheet 2", "Sheet 3"] and "Sheet 2" is the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet(&mut sheets, "Sheet 1");
        assert_next_sheet_is_active(&mut sheets, "Sheet 2", &pad);
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());

        // Assert now at version 1 created by "Set Active Sheet""
        assert!(ver_is(pad.version(), 1, "Set Active Sheet"));

        // Assert backward_versions is [(0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn set_active_sheet_invalid_id() {
        let mut master = WorkpadMaster::new_starter();

        // Try to switch to invalid sheet
        let sheet_id = SheetId(Version::MAX);
        let result = master.update(WorkpadUpdate::SetActiveSheet { sheet_id });

        assert!(result.is_err());
        assert_eq!(
            "SheetId(4294967295) not found (during update: Set Active Sheet)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn delete_active_sheet() {
        let mut master = WorkpadMaster::new_starter();

        // Delete the active sheet
        let sheet_id = master.active_version().active_sheet().unwrap().id();
        let pad = master
            .update(WorkpadUpdate::SheetDelete { sheet_id })
            .expect("Update should succeed");

        // Assert sheets is: ["Sheet 2", "Sheet 3"] and "Sheet 2" is now the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "Sheet 2", &pad);
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());

        // Assert now at version 1 created by "Delete Sheet""
        assert!(ver_is(pad.version(), 1, "Delete Sheet"));

        // Assert backward_versions is [(0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn delete_inactive_sheet() {
        let mut master = WorkpadMaster::new_starter();

        // Delete an inactive sheet
        let sheet_id = master.active_version().sheets().nth(1).unwrap().id();
        let pad = master
            .update(WorkpadUpdate::SheetDelete { sheet_id })
            .expect("Update should succeed");

        // Assert sheets is: ["Sheet 1", "Sheet 3"] and "Sheet 1" is still the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "Sheet 1", &pad);
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());

        // Assert now at version 1 created by "Delete Sheet""
        assert!(ver_is(pad.version(), 1, "Delete Sheet"));

        // Assert backward_versions is [(0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn delete_sheet_invalid_id() {
        let mut master = WorkpadMaster::new_starter();

        // Try to delete invalid sheet
        let sheet_id = SheetId(Version::MAX);
        let result = master.update(WorkpadUpdate::SheetDelete { sheet_id });

        assert!(result.is_err());
        assert_eq!(
            "SheetId(4294967295) not found (during update: Delete Sheet)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn add_sheets() {
        let mut master = WorkpadMaster::new_blank();

        // Add "New 1"
        let pad = master
            .update(WorkpadUpdate::SheetAdd {
                kind: SheetKind::Worksheet,
                name: String::from("New 1"),
            })
            .expect("Update should succeed");

        // Assert sheets is: ["New 1"] and "New 1" is the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "New 1", &pad);
        assert!(sheets.next().is_none());

        // Add "New 2"
        let pad = master
            .update(WorkpadUpdate::SheetAdd {
                kind: SheetKind::Worksheet,
                name: String::from("New 2"),
            })
            .expect("Update should succeed");

        // Assert sheets is: ["New 1", "New 2"] and "New 2" is now the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet(&mut sheets, "New 1");
        assert_next_sheet_is_active(&mut sheets, "New 2", &pad);
        assert!(sheets.next().is_none());

        // Assert now at version 2 created by "Add Sheet""
        assert!(ver_is(pad.version(), 2, "Add Sheet"));

        // Assert backward_versions is [(1, "Add Sheet"), (0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 1, "Add Sheet");
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    // TODO Should we be more restrictive than just not blank? (Valid names?)
    fn cannot_add_sheet_with_blank_name() {
        let mut master = WorkpadMaster::new_blank();
        let result = master.update(WorkpadUpdate::SheetAdd {
            kind: SheetKind::Worksheet,
            name: String::new(),
        });

        assert!(result.is_err());
        assert_eq!(
            r#"The name "" is not allowed (during update: Add Sheet)"#,
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn cannot_add_sheet_with_existing_name() {
        let mut master = WorkpadMaster::new_blank();
        master
            .update(WorkpadUpdate::SheetAdd {
                kind: SheetKind::Worksheet,
                name: String::from("New 1"),
            })
            .expect("Update should succeed");

        let result = master.update(WorkpadUpdate::SheetAdd {
            kind: SheetKind::Worksheet,
            name: String::from("New 1"),
        });

        assert!(result.is_err());
        assert_eq!(
            r#"The name "New 1" is already used (during update: Add Sheet)"#,
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn set_sheet_properties() {
        let mut master = WorkpadMaster::new_starter();

        // Set the sheet properties
        let sheet_id = master.active_version().active_sheet().unwrap().id();
        let pad = master
            .update(WorkpadUpdate::SheetSetProperties {
                sheet_id,
                new_name: String::from("Test Sheet"),
            })
            .expect("Update should succeed");

        // Assert sheets is: ["Test Sheet", "Sheet 2", "Sheet 3"] and "Test Sheet" is still the (renamed) active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "Test Sheet", &pad);
        assert_next_sheet(&mut sheets, "Sheet 2");
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());

        // Assert now at version 1 created by "Set Sheet Properties""
        assert!(ver_is(pad.version(), 1, "Set Sheet Properties"));

        // Assert backward_versions is [(0, "New Workpad"]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    // TODO Should we be more restrictive than just not blank? (Valid names?)
    fn cannot_set_sheet_properties_with_blank_name() {
        let mut master = WorkpadMaster::new_starter();

        // Set the sheet properties
        let sheet_id = master.active_version().active_sheet().unwrap().id();
        let result = master.update(WorkpadUpdate::SheetSetProperties {
            sheet_id,
            new_name: String::new(),
        });

        assert!(result.is_err());
        assert_eq!(
            r#"The name "" is not allowed (during update: Set Sheet Properties)"#,
            result.err().unwrap().to_string()
        );
    }

    #[test]
    // TODO Should we be more restrictive than just not blank? (Valid names?)
    fn cannot_set_sheet_properties_with_existing_name() {
        let mut master = WorkpadMaster::new_starter();

        // Set the sheet properties
        let sheet_id = master.active_version().active_sheet().unwrap().id();
        let result = master.update(WorkpadUpdate::SheetSetProperties {
            sheet_id,
            new_name: String::from("Sheet 2"),
        });

        assert!(result.is_err());
        assert_eq!(
            r#"The name "Sheet 2" is already used (during update: Set Sheet Properties)"#,
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn can_set_sheet_properties_to_its_current_name() {
        let mut master = WorkpadMaster::new_starter();

        // Set the sheet properties
        let sheet_id = master.active_version().active_sheet().unwrap().id();
        let pad = master
            .update(WorkpadUpdate::SheetSetProperties {
                sheet_id,
                new_name: String::from("Sheet 1"),
            })
            .expect("Update should succeed");

        // Assert sheets is: ["Sheet 1", "Sheet 2", "Sheet 3"] and "Sheet 1" is still the active sheet
        let mut sheets = pad.sheets();
        assert_next_sheet_is_active(&mut sheets, "Sheet 1", &pad);
        assert_next_sheet(&mut sheets, "Sheet 2");
        assert_next_sheet(&mut sheets, "Sheet 3");
        assert!(sheets.next().is_none());
    }

    #[test]
    fn set_sheet_active_cell() {
        let mut master = WorkpadMaster::new_starter();

        // Assert active cell is (0, 0) on all sheets
        let pad = master.active_version();
        let mut sheet_ids = vec![];
        for sheet in pad.sheets() {
            sheet_ids.push(sheet.id());
            match sheet.active_cell() {
                Some(cell) => {
                    assert_eq!(0, cell.row().index());
                    assert_eq!(0, cell.column().index());
                }
                None => panic!("Expected active cell"),
            }
        }

        // Change the active cell to (1, 1) for "Sheet 1"
        let sheet_id = sheet_ids[0];
        let sheet = pad.sheet_by_id(sheet_id).unwrap();
        let row_id = sheet.row(1).id();
        let column_id = sheet.column(1).id();
        master
            .update(WorkpadUpdate::SheetSetActiveCell {
                sheet_id,
                row_id,
                column_id,
            })
            .expect("Update should succeed");

        // Change the active cell to (2, 2) for "Sheet 2"
        let sheet_id = sheet_ids[1];
        let sheet = pad.sheet_by_id(sheet_id).unwrap();
        let row_id = sheet.row(2).id();
        let column_id = sheet.column(2).id();
        master
            .update(WorkpadUpdate::SheetSetActiveCell {
                sheet_id,
                row_id,
                column_id,
            })
            .expect("Update should succeed");

        // Change the active cell to (3, 3) for "Sheet 3"
        let sheet_id = sheet_ids[2];
        let sheet = pad.sheet_by_id(sheet_id).unwrap();
        let row_id = sheet.row(3).id();
        let column_id = sheet.column(3).id();
        master
            .update(WorkpadUpdate::SheetSetActiveCell {
                sheet_id,
                row_id,
                column_id,
            })
            .expect("Update should succeed");

        // Assert active cell is [(1, 1), (2, 2), (3, 3) on sheets ["Sheet 1", "Sheet 2", "Sheet 3"] respectively
        let pad = master.active_version();
        for (sheet, expected) in pad.sheets().zip(vec![(1, 1), (2, 2), (3, 3)]) {
            match sheet.active_cell() {
                Some(cell) => {
                    assert_eq!(expected.0, cell.row().index());
                    assert_eq!(expected.0, cell.column().index());
                }
                None => panic!("Expected active cell"),
            }
        }

        // Assert now at version 3 created by "Set Active Cell""
        assert!(ver_is(pad.version(), 3, "Set Sheet Active Cell"));

        // Assert backward_versions is [(2, "Set Sheet Active Cell"), (1, "Set Sheet Active Cell"), (0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 2, "Set Sheet Active Cell");
        assert_next_ver(&mut back_vers, 1, "Set Sheet Active Cell");
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn cannot_set_sheet_active_cell_to_unknown_sheet_id() {
        let mut master = WorkpadMaster::new_starter();
        let sheet = master.active_version().active_sheet().unwrap();
        let cell = sheet.cell(0, 0);

        // Try to set active cell with invalid sheet_id
        let result = master.update(WorkpadUpdate::SheetSetActiveCell {
            sheet_id: SheetId(Version::MAX),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
        });

        assert!(result.is_err());
        assert_eq!(
            "SheetId(4294967295) not found (during update: Set Sheet Active Cell)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn cannot_set_sheet_active_cell_to_unknown_row_id() {
        let mut master = WorkpadMaster::new_starter();
        let sheet = master.active_version().active_sheet().unwrap();
        let cell = sheet.cell(0, 0);

        // Try to set active cell with invalid row_id
        let result = master.update(WorkpadUpdate::SheetSetActiveCell {
            sheet_id: sheet.id(),
            row_id: RowId(Version::MAX),
            column_id: cell.column().id(),
        });

        assert!(result.is_err());
        assert_eq!(
            "RowId(4294967295) not found (during update: Set Sheet Active Cell)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn cannot_set_sheet_active_cell_to_unknown_column_id() {
        let mut master = WorkpadMaster::new_starter();
        let sheet = master.active_version().active_sheet().unwrap();
        let cell = sheet.cell(0, 0);

        // Try to set active cell with invalid column_id
        let result = master.update(WorkpadUpdate::SheetSetActiveCell {
            sheet_id: sheet.id(),
            row_id: cell.row().id(),
            column_id: ColumnId(Version::MAX),
        });

        assert!(result.is_err());
        assert_eq!(
            "ColumnId(4294967295) not found (during update: Set Sheet Active Cell)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn set_sheet_cell_value() {
        let mut master = WorkpadMaster::new_starter();

        // Assert active cell is empty
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.active_cell().unwrap();
        assert!(cell.value().is_empty());

        // Change the cell value
        let pad = master
            .update(WorkpadUpdate::SheetSetCellValue {
                sheet_id: sheet.id(),
                row_id: cell.row().id(),
                column_id: cell.column().id(),
                value: String::from("123"),
            })
            .expect("Update should succeed");

        // Assert new value
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.active_cell().unwrap();
        assert_eq!("123", cell.value());

        // Assert now at version 1 created by "Set Active Cell""
        assert!(ver_is(pad.version(), 1, "Set Sheet Cell Value"));

        // Assert backward_versions is [(0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn cannot_set_sheet_cell_value_on_unknown_sheet_id() {
        let mut master = WorkpadMaster::new_starter();
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.active_cell().unwrap();

        // Try to set cell value with invalid sheet_id
        let result = master.update(WorkpadUpdate::SheetSetCellValue {
            sheet_id: SheetId(Version::MAX),
            row_id: cell.row().id(),
            column_id: cell.column.id(),
            value: String::from("123"),
        });

        assert!(result.is_err());
        assert_eq!(
            "SheetId(4294967295) not found (during update: Set Sheet Cell Value)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn cannot_set_sheet_cell_value_on_unknown_row_id() {
        let mut master = WorkpadMaster::new_starter();
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.active_cell().unwrap();

        // Try to set cell value with invalid row_id
        let result = master.update(WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: RowId(Version::MAX),
            column_id: cell.column().id(),
            value: String::from("123"),
        });

        assert!(result.is_err());
        assert_eq!(
            "RowId(4294967295) not found (during update: Set Sheet Cell Value)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn cannot_set_sheet_cell_value_on_unknown_column_id() {
        let mut master = WorkpadMaster::new_starter();
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.active_cell().unwrap();

        // Try to set cell value with invalid column_id
        let result = master.update(WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell.row().id(),
            column_id: ColumnId(Version::MAX),
            value: String::from("123"),
        });

        assert!(result.is_err());
        assert_eq!(
            "ColumnId(4294967295) not found (during update: Set Sheet Cell Value)",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn multi_simple() {
        let mut master = WorkpadMaster::new_starter();

        // Assert active cell is empty
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell_0_0 = sheet.cell(0, 0);
        let cell_0_1 = sheet.cell(0, 1);
        let cell_0_2 = sheet.cell(0, 2);

        // Change the cell values
        let update_0_0 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell_0_0.row().id(),
            column_id: cell_0_0.column().id(),
            value: String::from("0"),
        };
        let update_0_1 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell_0_1.row().id(),
            column_id: cell_0_1.column().id(),
            value: String::from("1"),
        };
        let update_0_2 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell_0_2.row().id(),
            column_id: cell_0_2.column().id(),
            value: String::from("2"),
        };
        let pad = master
            .update(WorkpadUpdate::Multi(vec![
                update_0_0, update_0_1, update_0_2,
            ]))
            .expect("Update should succeed");

        // Assert new values
        let sheet = pad.active_sheet().unwrap();
        assert_eq!("0", sheet.cell(0, 0).value());
        assert_eq!("1", sheet.cell(0, 1).value());
        assert_eq!("2", sheet.cell(0, 2).value());

        // Assert now at version 1 created by "Set Active Cell""
        assert!(ver_is(
            pad.version(),
            1,
            "Set Sheet Cell Value & Set Sheet Cell Value & Set Sheet Cell Value"
        ),);

        // Assert backward_versions is [(0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn multi_update_same_cell_twice() {
        let mut master = WorkpadMaster::new_starter();

        // Assert active cell is empty
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.cell(0, 0);

        // Change the cell values
        let update_1 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
            value: String::from("0"),
        };
        let update_2 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
            value: String::from("1"),
        };
        let pad = master
            .update(WorkpadUpdate::Multi(vec![update_1, update_2]))
            .expect("Update should succeed");

        // Assert second value
        let sheet = pad.active_sheet().unwrap();
        assert_eq!("1", sheet.cell(0, 0).value());

        // Assert now at version 1 created by "Set Active Cell""
        assert!(ver_is(
            pad.version(),
            1,
            "Set Sheet Cell Value & Set Sheet Cell Value"
        ),);

        // Assert backward_versions is [(0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn multi_update_second_fails_so_all_fail() {
        let mut master = WorkpadMaster::new_starter();

        // Assert active cell is empty
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        let cell = sheet.cell(0, 0);

        // Change the cell values
        let update_1 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: sheet.id(),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
            value: String::from("0"),
        };
        let update_2 = WorkpadUpdate::SheetSetCellValue {
            sheet_id: SheetId(Version::MAX),
            row_id: cell.row().id(),
            column_id: cell.column().id(),
            value: String::from("1"),
        };
        let result = master.update(WorkpadUpdate::Multi(vec![update_1, update_2]));

        assert!(result.is_err());
        assert_eq!(
            "SheetId(4294967295) not found (during update: Set Sheet Cell Value)",
            result.err().unwrap().to_string()
        );

        // Assert first has not changed
        let pad = master.active_version();
        let sheet = pad.active_sheet().unwrap();
        assert!(sheet.cell(0, 0).value().is_empty());

        // Assert still at version 0 created by "New Workpad""
        assert!(ver_is(pad.version(), 0, "New Workpad"));

        // Assert no backward_versions and no forward_versions
        assert!(pad.backward_versions().next().is_none());
        assert!(pad.forward_versions().next().is_none());
    }

    #[test]
    fn switching_versions() {
        let mut master = WorkpadMaster::new_starter();

        let sheet = master
            .active_version()
            .active_sheet()
            .expect("Starter should have an active sheet");
        let cell = sheet.cell(0, 0);
        let mut set_value = |value| {
            master
                .update(WorkpadUpdate::SheetSetCellValue {
                    sheet_id: sheet.id(),
                    row_id: cell.row().id(),
                    column_id: cell.column().id(),
                    value,
                })
                .expect("Update should succeed")
        };

        // ================================================================================
        // Generate versions 1 & 2
        // ================================================================================
        let pad = set_value(String::from("A"));
        assert!(ver_is(pad.version(), 1, "Set Sheet Cell Value"));
        assert_eq!("A", pad.active_sheet().unwrap().cell(0, 0).value());

        let pad = set_value(String::from("B"));
        assert!(ver_is(pad.version(), 2, "Set Sheet Cell Value"));
        assert_eq!("B", pad.active_sheet().unwrap().cell(0, 0).value());

        // ================================================================================
        // Switch to version 1
        // ================================================================================
        let pad = pad
            .master()
            .update(WorkpadUpdate::SetVersion { version: 1 })
            .expect("Update should succeed");
        assert!(ver_is(pad.version(), 1, "Set Sheet Cell Value"));
        assert_eq!("A", pad.active_sheet().unwrap().cell(0, 0).value());

        // Assert backward_versions is [(0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert forward_versions is [(2, "Set Sheet Cell Value")]
        let mut forth_vers = pad.forward_versions();
        assert_next_ver(&mut forth_vers, 2, "Set Sheet Cell Value");
        assert!(forth_vers.next().is_none());

        // ================================================================================
        // Generate version 3 (which should be based on version 1)
        // ================================================================================
        let pad = set_value(String::from("C"));
        assert!(ver_is(pad.version(), 3, "Set Sheet Cell Value"));
        assert_eq!("C", pad.active_sheet().unwrap().cell(0, 0).value());

        // Assert backward_versions is [(1, "Set Sheet Cell Value"), (0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 1, "Set Sheet Cell Value");
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert no forward_versions
        assert!(pad.forward_versions().next().is_none());

        // ================================================================================
        // Switch to version 1 again - now its forward version should be 3
        // ================================================================================
        let pad = pad
            .master()
            .update(WorkpadUpdate::SetVersion { version: 1 })
            .expect("Update should succeed");
        assert!(ver_is(pad.version(), 1, "Set Sheet Cell Value"));
        assert_eq!("A", pad.active_sheet().unwrap().cell(0, 0).value());

        // Assert backward_versions is [(0, "New Workpad")]
        let mut back_vers = pad.backward_versions();
        assert_next_ver(&mut back_vers, 0, "New Workpad");
        assert!(back_vers.next().is_none());

        // Assert forward_versions is [(2, "Set Sheet Cell Value")]
        let mut forth_vers = pad.forward_versions();
        assert_next_ver(&mut forth_vers, 3, "Set Sheet Cell Value");
        assert!(forth_vers.next().is_none());
    }

    fn assert_next_ver(
        iter: &mut impl Iterator<Item = (Version, String)>,
        expected_version: Version,
        expected_desc: &str,
    ) {
        match iter.next() {
            Some(ver) => assert!(ver_is(ver, expected_version, expected_desc)),
            None => panic!(r#"Expected ({}, "{}")"#, expected_version, expected_desc),
        };
    }

    fn ver_is(ver: (Version, String), expected_version: Version, expected_desc: &str) -> bool {
        ver.0 == expected_version && ver.1 == expected_desc
    }

    fn assert_next_sheet(iter: &mut impl Iterator<Item = Sheet>, expected_name: &str) {
        match iter.next() {
            Some(s) => assert_eq!(expected_name, s.name()),
            None => panic!(r#"Expected: "{}""#, expected_name),
        };
    }

    fn assert_next_sheet_is_active(
        iter: &mut impl Iterator<Item = Sheet>,
        expected_name: &str,
        pad: &Workpad,
    ) {
        match iter.next() {
            Some(s) => {
                assert_eq!(expected_name, s.name());
                assert_eq!(pad.active_sheet().unwrap(), s);
            }
            None => panic!(r#"Expected: "{}""#, expected_name),
        };
    }
}
