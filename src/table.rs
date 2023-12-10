use std::collections::HashMap;
use std::io;
use std::slice::{Iter, IterMut};

use csv::StringRecord;

#[derive(Debug)]
pub struct Table {
    name: String,
    records: Vec<Record>,
}

/// A Variables is a map of field: value, which are all Stringly typed
pub type Record = HashMap<String, String>;

pub fn from_csv<R: io::Read>(name: String, reader: R) -> anyhow::Result<Table> {
    let mut vars_csv = csv::Reader::from_reader(reader);

    let fields: Vec<_> = vars_csv.headers()?.iter().map(str::to_string).collect();

    let map_record = |result: Result<StringRecord, csv::Error>| {
        result.map(|record| {
            record
                .iter()
                .enumerate()
                .map(|(n, value)| (fields[n].to_owned(), value.to_owned()))
                .collect::<HashMap<String, String>>()
        })
    };

    let records = vars_csv
        .records()
        .map(map_record)
        .collect::<Result<Vec<Record>, _>>()?;

    Ok(Table::new(name, records))
}

impl Table {
    pub fn new(name: String, records: Vec<Record>) -> Self {
        Table { name, records }
    }

    pub fn index(&self, index: &str) -> anyhow::Result<Option<&Record>> {
        if self.records.is_empty() {
            return Ok(None);
        }

        if !self.records[0].contains_key("$id") {
            anyhow::bail!(format!("Invalid table `{}` has no $id field", &self.name))
        }

        Ok(self
            .records
            .iter()
            .find(|record| record.get("$id").unwrap() == index))
    }

    pub fn iter(&self) -> Iter<'_, Record> {
        self.records.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Record> {
        self.records.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::build_test_table;

    #[test]
    fn test_table_index() -> anyhow::Result<()> {
        let fields = &["$id", "bar", "baz quux"];
        let table = build_test_table("test table", fields, 5);

        let record = table.index("$id #3")?.unwrap();
        assert_eq!(record.get("bar"), Some(&String::from("bar #3")));

        let record = table.index("$id #8")?;
        assert_eq!(record, None);

        let record = table.index("7")?;
        assert_eq!(record, None);

        let fields = &["no $id field"];
        let table = build_test_table("missing $id", fields, 3);

        let Err(err) = table.index("$id #3") else {
            panic!("expected error!");
        };

        assert_eq!(
            err.to_string(),
            String::from("Invalid table `missing $id` has no $id field")
        );

        Ok(())
    }
}
