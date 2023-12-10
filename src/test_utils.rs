use std::collections::HashMap;

use crate::Table;

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
