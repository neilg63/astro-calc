use libswe_sys::sweconst::{Bodies};

pub trait FromKey<T> {
  fn from_key(&self, key: &str) -> T;
}

impl FromKey<Bodies> for Bodies {
  fn from_key(&self, key: &str) -> Bodies {
    let simple_key = key.to_lowercase();
    match simple_key.as_str() {
      "su" => Bodies::Sun,
      "mo" => Bodies::Moon,
      "me" => Bodies::Mercury,
      "ve" => Bodies::Venus,
      "ea" => Bodies::Earth,
      "ma" => Bodies::Mars,
      "ju" => Bodies::Jupiter,
      "sa" => Bodies::Saturn,
      "ne" => Bodies::Neptune,
      "ur" => Bodies::Uranus,
      "pl" => Bodies::Pluto,
      "ke" | "ra" => Bodies::TrueNode,
      "mn" => Bodies::MeanNode,
      "kr" => Bodies::Kronos,
      "is" => Bodies::Isis,
      "jn" => Bodies::Juno,
      "ce" => Bodies::Ceres,
      "ch" => Bodies::Chiron,
      "sn" => Bodies::SouthNode,
      _ => Bodies::Earth,
    }
  }
}
