use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result};
use super::super::traits::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Ayanamsha {
  Tropical = 0,
  TrueCitra = 27,
  Lahiri = 1,
  Krishnamurti = 5,
  Yukteshwar = 7,
  Raman = 3,
  ValensMoon = 42,
  TrueMula = 35,
  TrueRevati = 28,
  TruePushya = 29,
  TrueSheoran = 39,
  Aldebaran15Tau = 14,
  GalcentMulaWilhelm = 36,
  GalcentCochrane = 40,
  Hipparchos = 15,
  Sassanian = 16,
  Ushashashi = 4,
  JnBhasin = 8,
}

impl Ayanamsha {
  
  pub fn as_string(&self) -> String {
    match self {
      Ayanamsha::TrueCitra => "true_citra",
      Ayanamsha::Lahiri => "lahiri",
      Ayanamsha::Krishnamurti => "krishnamurti",
      Ayanamsha::Yukteshwar => "yukteshwar",
      Ayanamsha::Raman => "raman",
      Ayanamsha::ValensMoon => "valensmoon",
      Ayanamsha::TrueMula => "true_mula",
      Ayanamsha::TrueRevati => "true_revati",
      Ayanamsha::TruePushya => "true_pushya",
      Ayanamsha::TrueSheoran => "true_sheoran",
      Ayanamsha::Aldebaran15Tau => "aldebaran_15_tau",
      Ayanamsha::GalcentMulaWilhelm => "galcent_mula_wilhelm",
      Ayanamsha::GalcentCochrane => "galcent_cochrane",
      Ayanamsha::Hipparchos => "hipparchos",
      Ayanamsha::Sassanian => "sassanian",
      Ayanamsha::Ushashashi => "ushashashi",
      Ayanamsha::JnBhasin => "jnbhasin",
      _ => "tropical",
    }.to_string()
  }

  pub fn as_i32(self) -> i32 {
    self as i32
  }
}

impl Display for Ayanamsha {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}", self.as_string())
  } 
}

impl FromKey<Ayanamsha> for Ayanamsha {
  fn from_key(key: &str) -> Self {
    let simple_str = key.to_lowercase().replace("_", "");
    match simple_str.as_str() {
      "tc" | "truecitra" | "true_citra" | "citra" | "chitra" => Ayanamsha::TrueCitra,
      "lh" | "lahiri" => Ayanamsha::Lahiri,
      "kr" | "krishnamurti" => Ayanamsha::Krishnamurti,
      "yu" | "yukteshwar" => Ayanamsha::Yukteshwar,
      "ra" | "raman" => Ayanamsha::Raman,
      "vm" | "valensmoon" => Ayanamsha::ValensMoon,
      "tm" | "truemula" => Ayanamsha::TrueMula,
      "tr" | "truerevati" => Ayanamsha::TrueRevati,
      "tp" | "truepushya" | "pushya" => Ayanamsha::TruePushya,
      "ts" | "truesheoran" => Ayanamsha::TrueSheoran,
      "at" | "aldebaran15tau" => Ayanamsha::Aldebaran15Tau,
      "gm" | "galcenmulawilhelm" => Ayanamsha::GalcentMulaWilhelm,
      "gc" | "galcentcochrane" => Ayanamsha::GalcentCochrane,
      "hi" | "hipparchos" => Ayanamsha::Hipparchos,
      "sa" | "sassanian" => Ayanamsha::Sassanian,
      "us" | "ushashashi" => Ayanamsha::Ushashashi,
      "jb" | "jnbhasin" => Ayanamsha::JnBhasin,
      _ => Ayanamsha::Tropical,
    }
  }
}

pub fn all_ayanamsha_keys() -> Vec<&'static str> {
  vec![
    "true_citra",
    "lahiri",
    "krishnamurti",
    "yukteshwar",
    "raman",
    "valensmoon",
    "true_mula",
    "true_revati",
    "true_pushya",
    "true_sheoran",
    "aldebaran_15_tau",
    "galcent_mula_wilhelm",
    "galcent_cochrane",
    "hipparchos",
    "sassanian",
    "ushashashi",
    "jnbhasin",
  ]
}
