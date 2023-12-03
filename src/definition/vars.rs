use std::{collections::HashMap, io};

use super::field::Field;

#[derive(Debug)]
pub struct Vars {
    pub fields: Vec<Field>,
    pub values: Vec<VarSet>,
}

/// A set of variables
#[derive(Debug, Clone)]
pub struct VarSet(pub HashMap<Field, String>);

#[derive(Debug)]
pub struct Type {
    /// The name of the type, e.g. job
    pub name: String,

    /// The set of variables belonging to this type
    pub vars: Vars,
}

impl Vars {
    pub fn new(fields: Vec<Field>, values: Vec<VarSet>) -> Self {
        Vars { fields, values }
    }

    pub fn from_csv<R: io::Read>(reader: R) -> anyhow::Result<Self> {
        let mut vars_csv = csv::Reader::from_reader(reader);

        let fields: Vec<Field> = vars_csv.headers()?.into_iter().map(Field::new).collect();

        Ok(vars_csv
            .records()
            .map(|result| {
                result.map(|record| {
                    VarSet(
                        record
                            .iter()
                            .enumerate()
                            .map(|(n, value)| (fields[n].clone(), value.to_owned()))
                            .collect::<HashMap<Field, String>>(),
                    )
                })
            })
            .collect::<Result<Vec<VarSet>, _>>()
            .map(|values| Vars::new(fields, values))?)
    }
}

impl Type {
    pub fn new(name: String, vars: Vars) -> Self {
        Type { name, vars }
    }

    pub fn from_csv<R: io::Read>(reader: R, type_name: String) -> anyhow::Result<Self> {
        let mut type_csv = csv::Reader::from_reader(reader);

        let fields: Vec<Field> = type_csv.headers()?.into_iter().map(Field::new).collect();

        Ok(type_csv
            .records()
            .map(|result| {
                result.map(|record| {
                    VarSet(
                        record
                            .iter()
                            .enumerate()
                            .map(|(n, value)| (fields[n].clone(), value.to_owned()))
                            .collect::<HashMap<Field, String>>(),
                    )
                })
            })
            .collect::<Result<Vec<VarSet>, _>>()
            .map(|values| Type::new(type_name, Vars::new(fields, values)))?)
    }
}
