use async_dup::Arc;
use camino::Utf8Path;
use krill_common::KrillResult;
use redb::{Database, Error, ReadableTable, TableDefinition};

type AllBytesTableDefinition = TableDefinition<'static, &'static [u8], Vec<u8>>;

pub struct KrillClientStorage {
    db: Arc<Database>,
    secrets: AllBytesTableDefinition,
}

impl KrillClientStorage {
    pub fn new(path: &Utf8Path) -> KrillResult<Self> {
        let db = Arc::new(Database::create(path).map_err(|error| {
            let error: redb::Error = error.into();

            error
        })?);
        let secrets = TableDefinition::new("SECRETS_TABLE");

        Ok(Self { db, secrets })
    }

    pub fn store(&self) -> Arc<Database> {
        self.db.clone()
    }

    pub fn secrets_table(&self) -> AllBytesTableDefinition {
        self.secrets
    }
}

// fn main() -> Result<(), Error> {
//     let db = Database::create("my_db.redb")?;
//     let write_txn = db.begin_write()?;
//     {
//         let mut table = write_txn.open_table(TABLE)?;
//         table.insert("my_key", &123)?;
//     }
//     write_txn.commit()?;

//     let read_txn = db.begin_read()?;
//     let table = read_txn.open_table(TABLE)?;
//     assert_eq!(table.get("my_key")?.unwrap().value(), 123);

//     Ok(())
// }
