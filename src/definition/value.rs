// /// Values are either a plain String or a reference to Type by its $id
// #[derive(Debug)]
// pub enum Value {
//     TypeId(String),
//     Plain(String),
// }

// impl Value {
//     pub fn upgrade_to_type(self) -> Self {
//         match self {
//             Value::Plain(s) => Value::TypeId(s),
//             already_a_type_id => already_a_type_id,
//         }
//     }
// }

// impl AsRef<str> for Value {
//     fn as_ref(&self) -> &str {
//         match self {
//             Value::Plain(s) => s.as_ref(),
//             Value::TypeId(s) => s.as_ref(),
//         }
//     }
// }
