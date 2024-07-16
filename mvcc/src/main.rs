use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct TableRow {
    id: u32,
    name: String,
}

pub struct TableStore {
    rows: Vec<TableRow>,
}

// A globally incrementing version number.
static VERSION: AtomicUsize = AtomicUsize::new(1);

// Function to acquire the next version number.
fn acquire_next_version() -> usize {
    VERSION.fetch_add(1, Ordering::SeqCst)
}

lazy_static! {
    // Stores the currently active transaction IDs along with the versions they have written.
    static ref ACTIVE_TXN: Arc<Mutex<HashMap<usize, Vec<(u32, String)>>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Definition of an MVCC (Multi-Version Concurrency Control) transaction.
pub struct MVCC {
    table: Arc<Mutex<TableStore>>,
}

impl MVCC {
    // Constructor for creating a new MVCC instance.
    pub fn new(table: TableStore) -> Self {
        Self {
            table: Arc::new(Mutex::new(table)),
        }
    }

    // Begin a new transaction.
    pub fn begin_transaction(&self) -> Transaction {
        Transaction::begin(self.table.clone())
    }
}

// Representation of an MVCC transaction.
pub struct Transaction {
    // The underlying table store.
    table: Arc<Mutex<TableStore>>,
    // The version number assigned to this transaction.
    version: usize,
    // A list of active transaction IDs at the time the transaction was started.
    active_xids: HashSet<usize>,
}

impl Transaction {
    // Start a new transaction.
    pub fn begin(table: Arc<Mutex<TableStore>>) -> Self {
        // Obtain a global version number for the transaction.
        let version = acquire_next_version();

        let mut active_txns = ACTIVE_TXN.lock().unwrap();
        // Collect all currently active transaction IDs.
        let active_xids = active_txns.keys().cloned().collect();

        // Add the current transaction ID to the list of active transactions.
        active_txns.insert(version, Vec::new());

        // Return the initialized transaction.
        Self {
            table,
            version,
            active_xids,
        }
    }

    // Write data to the database within the scope of the transaction.
    pub fn set(&self, id: u32, name: String) {
        self.write(id, Some(name));
    }

    // Delete data from the database within the scope of the transaction.
    pub fn delete(&self, id: u32) {
        self.write(id, None);
    }

    // Internal method to perform write operations.
    fn write(&self, id: u32, name: Option<String>) {
        let mut table = self.table.lock().unwrap();
        match name {
            Some(n) => {
                // Find the index of the row with the given ID.
                let idx = table.rows.iter().position(|r| r.id == id);
                if let Some(idx) = idx {
                    // Replace the existing row with the new name.
                    table.rows[idx] = TableRow { id, name: n };
                } else {
                    // Insert a new row if the ID doesn't exist.
                    table.rows.push(TableRow { id, name: n });
                }
            }
            None => {
                // Remove the row with the given ID.
                table.rows.retain(|r| r.id != id);
            }
        }
    }

    // Read data from the database, starting from the most recent version and stopping at the first visible one.
    pub fn get(&self, id: u32) -> Option<String> {
        let table = self.table.lock().unwrap();
        for row in &table.rows {
            if row.id == id && self.is_visible(version) {
                return Some(row.name.clone());
            }
        }
        None
    }

    // Commit the transaction, removing it from the list of active transactions.
    pub fn commit(&self) {
        let mut active_txns = ACTIVE_TXN.lock().unwrap();
        active_txns.remove(&self.version);
    }

    // Rollback the transaction, undoing any writes made during the transaction.
    pub fn rollback(&self) {
        let mut active_txns = ACTIVE_TXN.lock().unwrap();
        if let Some(entries) = active_txns.get(&self.version) {
            let mut table = self.table.lock().unwrap();
            for (id, name) in entries {
                // Restore the state of the table to before the transaction.
                table.rows.retain(|r| r.id != *id);
            }
        }
        active_txns.remove(&self.version);
    }

    // Determine whether a version of data is visible to the current transaction.
    fn is_visible(&self, version: usize) -> bool {
        if self.active_xids.contains(&version) {
            return false;
        }
        version <= self.version
    }
}

fn main() {
    // Initialize the table store.
    let table_store = TableStore { rows: Vec::new() };

    // Create an instance of the MVCC system using the initialized table store.
    let mvcc = MVCC::new(table_store);

    // Start a new transaction.
    let transaction1 = mvcc.begin_transaction();

    // Perform set operations within the transaction.
    transaction1.set(1, "Alice".into());
    transaction1.set(2, "Bob".into());
    transaction1.set(3, "Charlie".into());

    // Print the current state of the table store to verify the set operations.
    println!("After Transaction1 sets:");
    for row in &mvcc.table.lock().unwrap().rows {
        println!("ID: {}, Name: {}", row.id, row.name);
    }

    // Start another transaction.
    let transaction2 = mvcc.begin_transaction();

    // Perform a delete operation within the second transaction.
    transaction2.delete(2);

    // Print the current state of the table store to verify the delete operation.
    println!("After Transaction2 deletes ID 2:");
    for row in &mvcc.table.lock().unwrap().rows {
        println!("ID: {}, Name: {}", row.id, row.name);
    }

    // Commit the first transaction.
    transaction1.commit();

    // Verify that the commit makes the changes visible to subsequent transactions.
    let transaction3 = mvcc.begin_transaction();
    println!("After Transaction1 commits, Transaction3 sees:");
    for row in &mvcc.table.lock().unwrap().rows {
        println!("ID: {}, Name: {}", row.id, row.name);
    }

    // Attempt to roll back the second transaction.
    transaction2.rollback();

    // Verify that the rollback undoes the delete operation.
    println!("After Transaction2 rolls back, the table state is:");
    for row in &mvcc.table.lock().unwrap().rows {
        println!("ID: {}, Name: {}", row.id, row.name);
    }

    // Clean up the MVCC instance.
    drop(mvcc);
}
