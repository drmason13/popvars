// // expr expansion

// // // in scope:
// // // * variable - the row from vars currently being templated
// // // * types - all the types available

// // $(field) -> variable.get(field) -> find the value for the field

// // $(typ.field) -> {
// //     typ = types.get(typ);
// //     var_value = variable.get(typ);
// //     typ_value = typ.find(|var| var.get("$id") == var_value);
// //     typ_value.get(field)
// // } -> find the typ, find the variable in typ that matches the
// // variable's value of the field that has the same name as the typ

// pub enum Expr {
//     Direct(String),
//     Lookup(Lookup),
// }

// pub struct Lookup<const N: usize> {
//     get: String,
//     from: Vec<String>,
//     via: Option<String>,
// }

// "country.code" => "SELECT code "
//                   "FROM country JOIN vars ON country.$id = vars.country "
//                   "WHERE vars.id = $CURR;"

// "country" => Expr::Lookup {

// }

// "country.code" => Expr::Lookup {
//     get: "code",
//     from: Some(vec!["country"]),
//     via: None,
// }

// "Enemy Country@country.code" => Expr::Lookup {
//     get: "code",
//     from: Some(vec!["country"]),
//     via: Some("Enemy Country"),
// }

// "country.team.code" => Expr::Lookup {
//     get: "code",
//     from: Some(vec!["country", "team"]),
//     via: None,
// }

// "Enemy Country@country.team.code" => Expr::Lookup {
//     get: "code",
//     from: Some(vec!["country", "team"]),
//     via: Some("Enemy Country"),
// }

// "Enemy Country@country.enemy@team.code" => Expr::Lookup {
//     get: "code",
//     from: Some(vec!["country", "team"]),
//     via: Some("Enemy Country"),
// }

// // https://regex101.com/r/dUbsJa/1

// $(country)
// $(country.code)
// $(country@Enemy Country.code)
// $(country.team.code)
// $(country@Enemy Country.team.code)

// /// Leads to a stack of [`Context`]s that either lookup a new context `table(field)` by getting the value of `field` in the current context and using it to index `table`
// /// country.team is then just a shortcut for country(country).team where the field and the table share the same name!

// pub type Context = HashMap<String, String>; // i.e. a Record

// // maybe a (mutable) shared ref to a Context. We can update where this ctx points to as we go... maybe! Will see what compiler thinks
// let mut ctx: &Context;

// lookup(table, key) -> replace the current context with a new one found by indexing Table `table` using a value found at Field `key`
// index(table, value) -> return the Record within Table `table` where record.get("$id") == "key"

// `Table`s are indexed by *value* -> return the Record within Table `table` where record.get("$id") == value -> record =
// `Record`s are indexed by *key* -> value = record.get(key)

// a key refers to a value that is _presumed_ to appear once in a table in the "$id" field, it is used to index a unique record from that table
// a field is a String used to index a Record


// overall parser progress
// $(country@Enemy Country.team.code)
// country@Enemy Country.team.code