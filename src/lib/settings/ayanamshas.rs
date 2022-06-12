use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result};

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
  pub fn from_key(key: &str) -> Self {
    let simple_str = key.to_lowercase().replace("_", "");
    match simple_str.as_str() {
      "truecitra" | "citra" | "chitra" => Ayanamsha::TrueCitra,
      "lahiri" => Ayanamsha::Lahiri,
      "krishnamurti" => Ayanamsha::Krishnamurti,
      "yukteshwar" => Ayanamsha::Yukteshwar,
      "raman" => Ayanamsha::Raman,
      "valensmoon" => Ayanamsha::ValensMoon,
      "truemula" => Ayanamsha::TrueMula,
      "truerevati" => Ayanamsha::TrueRevati,
      "truepushya" | "pushya" => Ayanamsha::TruePushya,
      "truesheoran" => Ayanamsha::TrueSheoran,
      "aldebaran15tau" => Ayanamsha::Aldebaran15Tau,
      "galcenmulawilhelm" => Ayanamsha::GalcentMulaWilhelm,
      "galcentcochrane" => Ayanamsha::GalcentCochrane,
      "hipparchos" => Ayanamsha::Hipparchos,
      "sassanian" => Ayanamsha::Sassanian,
      "ushashashi" => Ayanamsha::Ushashashi,
      "jnbhasin" => Ayanamsha::JnBhasin,
      _ => Ayanamsha::Tropical,
    }
  }

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
