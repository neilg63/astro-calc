/*
* Serves to match &str keys to enum types
*/
pub trait FromKey<T> {
  fn from_key(key: &str) -> T;
}

pub trait ToKey<T> {
  fn to_key(&self) -> &str;
}

pub trait ToISODateString {
  fn iso_date_string(&self) -> String;
}