- input: 
convert! {
   Email => { self.value.clone() } 
}
- output:
 match target_config {
     FieldConfig::Text(config) => match config {
         TextConfig::Email => Ok(Value::Email(Email {
             value: self.value.clone(),
         })),
         _ => Err(ValueError::WrongType(
             "cant convert to this type".to_string(),
         )),
     },
     _ => Err(ValueError::WrongType(
         "cant convert to this type".to_string(),
     )),
 }

