use std::collections::HashMap;
use std::io;
use std::slice::{Iter, IterMut};

use csv::StringRecord;

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub records: Vec<Record>,
}

/// Fields are just Strings. They are appear in a table header.
/// Fields are used to name and refer to the values in a [Record]
pub type Field = String;

/// Values are just Strings. They are appear in a table rows, outside of the header.
/// Values appear in a template after it has been populated.
pub type Value = String;

/// A Record is a map of [field]: [value], both of which are actually just `String`.
///
/// Records are rows in a [table]. Each key in the hashmap is a field in the Record.
///
/// [table]: Table
/// [field]: Field
/// [value]: Value
pub type Record = HashMap<Field, Value>;

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
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Table::new(name, records))
}

impl Table {
    pub fn new(name: String, records: Vec<Record>) -> Self {
        Table { name, records }
    }

    pub fn index<'c>(&'c self, index: &'c str) -> anyhow::Result<Option<&'c Record>> {
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
    pub use super::*;

    use std::collections::HashMap;

    pub fn build_test_table(name: &str, fields: &[&str], len: usize) -> Table {
        let mut records = Vec::new();

        let record = |idx| {
            let mut map = HashMap::<String, String>::new();
            for field in fields {
                map.insert(field.to_string(), format!("{field} #{idx}"));
            }
            map
        };

        for idx in 0..len {
            records.push(record(idx));
        }

        Table::new(name.to_string(), records)
    }

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
