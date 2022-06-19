
pub struct VargaItem {
  pub num: u8,
  pub parashari: bool,
  pub parivritti: bool,
  pub tajik: bool,
  pub nadi: bool,
}

impl VargaItem {
  pub fn new(num: u8, parashari: bool, parivritti: bool, tajik: bool, nadi: bool) -> VargaItem {
    VargaItem { num: num, parashari: parashari, parivritt, nadi }
  }
}

pub const VARGA_VALUES: Vec<VargaItem> = vec![
  VargaItem::new(1, false, true, true, true),
  VargaItem::new(2, false, true, true, true),
  VargaItem::new(3, false, true, true, true),
  VargaItem::new(4, false, true, true, true),
  VargaItem::new(7, false, true, true, true),
  VargaItem::new(9, false, true, true, true),
  VargaItem::new(10, false, true, true, true),
  VargaItem::new(12, false, true, true, true),
  VargaItem::new(16, false, true, true, true),
  VargaItem::new(20, false, true, true, true),
  VargaItem::new(24, false, true, true, true),
  VargaItem::new(27, false, true, true, true),
  // (Standard Parāśara, Parivṛtti (cyclical), Ṣaṣṭyaṁśa like Triṁṣāṁśa)
  VargaItem::new(30, false, true, true, true),
  VargaItem::new(40, false, true, true, true),
  VargaItem::new(45, false, true, true, true),
  VargaItem::new(60, false, true, true, true),
  VargaItem::new(5, false, false, true, true),
  VargaItem::new(8, false, false, true, true),
  // (Parivṛtti (cyclical) D8/ H8, Continuous & regular, Tattva based Discontinuous Dṛ. B.V. Raman)
  VargaItem::new(11, false, false, true, true),
  VargaItem::new(72, false, false, true, true),
  VargaItem::new(81, false, false, true, true),
  VargaItem::new(108, false, false, true, true),
  VargaItem::new(144, false, false, true, true),
  VargaItem::new(150, false, false, true, true),
  VargaItem::new(300, false, false, true, true),
];