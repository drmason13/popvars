pub struct Template {}

#[allow(dead_code)]
pub enum Expression {
    Value(String),
    Dot(String, Box<Expression>),
}
