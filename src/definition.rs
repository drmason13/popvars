mod field;
mod value;
mod vars;

use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use erreport::path::ErrorPaths;

use crate::template::Template;

use self::vars::{Type, Vars};

#[derive(Debug)]
pub struct Definition {
    /// Each VarSet in Vars is used to populate a template
    pub vars: Vars,

    /// Types are keyed by name so they can easily be retrieved
    pub types: HashMap<String, Type>,
}

impl Definition {
    pub fn from_ods_file<P: AsRef<Path>>(path: P) -> Self {
        let _ = path;
        todo!("read definitions from ods file")
    }

    /// vars is required, types may be empty. Strings are expected to be in csv format.
    pub fn from_csv_files(vars: &Path, types: &[PathBuf]) -> anyhow::Result<Self> {
        let types = types
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
                    name_res.map(|name| match Type::from_csv(reader, name.to_string()) {
                        Ok(type_) => Ok((name.to_string(), type_)),
                        Err(e) => Err(anyhow::anyhow!(e)),
                    })
                })
            })
            .collect::<Result<Result<Result<HashMap<String, Type>, _>, _>, _>>()???;

        let vars = File::open(vars)
            .path(vars)
            .map(BufReader::new)
            .map(Vars::from_csv)??;

        let definition = Definition { vars, types };

        Ok(definition)
    }

    pub fn render(&self, template: Template) -> String {
        let _ = template;
        // loop through the template and replace each interpolation with the values we have defined

        // dot expressions (typeId.field) should look up their values from the field of the type instance with id typeId.
        todo!()
    }
}
