use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use erreport::path::ErrorPaths;

use crate::table::{self, Table};

#[derive(Debug)]
pub struct Definition {
    /// This [table] is used to populate the template
    ///
    /// [table]: Table
    pub vars: Table,

    /// defs are [table]s keyed by name that can be used to resolve [lookups] during an [expansion]
    ///
    /// [table]: Table
    /// [lookups]: crate::expr::Lookup
    /// [expansion]: crate::expr::Expand
    pub defs: HashMap<String, Table>,
}

impl Definition {
    pub fn from_ods_file<P: AsRef<Path>>(path: P) -> Self {
        let _ = path;
        todo!("read definitions from ods file")
    }

    /// vars is required, defs may be empty. Strings are expected to be in csv format.
    pub fn from_csv_files(vars: &Path, defs: &[PathBuf]) -> anyhow::Result<Self> {
        let vars = File::open(vars)
            .path(vars)
            .map(BufReader::new)
            .map(|reader| table::from_csv("vars".to_owned(), reader))??;

        let defs = defs
            .iter()
            .map(|path| {
                (
                    File::open(path).path(path).map(BufReader::new),
                    path.file_prefix()
                        .ok_or_else(|| {
                            anyhow::anyhow!(format!(
                                "path `{}` is missing a filename, which is needed to name the type",
                                path.display()
                            ))
                        })
                        .map(|os_str| os_str.to_string_lossy()),
                )
            })
            .map(|(reader_res, name_res)| {
                reader_res.map(|reader| {
                    name_res.map(|name| match table::from_csv(name.to_string(), reader) {
                        Ok(type_) => Ok((name.to_string(), type_)),
                        Err(e) => Err(anyhow::anyhow!(e)),
                    })
                })
            })
            .collect::<Result<Result<Result<HashMap<String, Table>, _>, _>, _>>()???;

        let definition = Definition { vars, defs };

        Ok(definition)
    }
}
