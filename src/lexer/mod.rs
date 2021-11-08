
#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Number(String),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub enum Token {
    AnonymousRecord,
    Column(String),
    NamedRecord(String),
    Schema(String),
    Table(String),
    Value(Box<Value>),
}

pub fn lex(text: &str) -> Vec<Token> {
    vec![]
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_good_file() {
        use super::{
            Token as T,
            Value as V,
            lex,
        };

        let file =
"public
  -- This is a newline comment
  pet
    cupid
      name 'Cupid' -- This is a trailing comment
      species 'cat'
      lap_cat true

    _
      name 'Eiyre'
      lap_cat false

  person
    kevin
      name 'Kevin'
      age 38

schema1:
  message:
    _:
      text 'Hello, world!'
";

        assert_eq!(lex(file), vec![
            T::Schema("public".to_owned()),
                T::Table("pet".to_owned()),
                    T::NamedRecord("cupid".to_owned()),
                        T::Column("name".to_owned()),
                        T::Value(Box::new(V::Text("Cupid".to_owned()))),
                        T::Column("species".to_owned()),
                        T::Value(Box::new(V::Text("cat".to_owned()))),
                        T::Column("lap_cat".to_owned()),
                        T::Value(Box::new(V::Boolean(true))),
                    T::AnonymousRecord,
                        T::Column("name".to_owned()),
                        T::Value(Box::new(V::Text("Eiyre".to_owned()))),
                        T::Column("lap_cat".to_owned()),
                        T::Value(Box::new(V::Boolean(false))),
                T::Table("person".to_owned()),
                    T::NamedRecord("kevin".to_owned()),
                        T::Column("name".to_owned()),
                        T::Value(Box::new(V::Text("Kevin".to_owned()))),
                        T::Column("age".to_owned()),
                        T::Value(Box::new(V::Number("38".to_owned()))),
            T::Schema("schema1".to_owned()),
                T::Table("message".to_owned()),
                    T::AnonymousRecord,
                        T::Column("text".to_owned()),
                        T::Value(Box::new(V::Text("Hello, world!".to_owned()))),
        ]);
    }
}
