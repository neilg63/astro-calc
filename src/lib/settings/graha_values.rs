use libswe_sys::sweconst::{Bodies};
use super::super::traits::*;
use serde::{Serialize, Deserialize};

impl FromKey<Bodies> for Bodies {
  fn from_key(key: &str) -> Bodies {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrahaInfo {
  num: u8,
  jy_num: u8,
  key: String,
  icon: String,
  nature: Vec<String>,
  gender: char,
  bhuta: String,
  guna: String,
  caste: u8,
  dhatu: u8,
  dosha: Vec<String>,
  friends: Vec<String>,
  neutral: Vec<String>,
  enemies: Vec<String>,
  #[serde(rename="ownSign")]
  own_sign: Vec<u8>,
  #[serde(rename="exaltedDegree")]
  exalted_degree: u16,
  #[serde(rename="mulaTrikon")]
  mula_trikon: u8,
  #[serde(rename="mulaTrikonDegrees")]
  mula_trikon_degrees: Vec<u16>,
  #[serde(rename="charaKarakaReverse")]
  chara_karaka_reverse: bool,
}