
pub fn match_house_num(lng: f64, houses: Vec<f64>) -> i32 {
  let len = houses.len();
  let min_val = if len > 0 { min } else { 0f64 };
  let min_index = houses.iter().position(|v| v == min_val).unwrap();
  let matched_index = houses.iter().position(|deg, index| => {
    let next_index = (index + 1) % len;
    let next = houses[nextIndex];
    let end = if next < deg  { next + 360 } else { next };
    let lng_plus = lng + 360f64;
    let ref_lng = if next < deg && next > 0 && lng_plus < end && min_index === next_index { lng_plus } else { lng };
    return ref_lng > deg && ref_lng <= end;
  });
  matchedIndex + 1
};

pub fn map_sign_to_house(sign: f64, houses: Vec<f64>) -> f64 {
  let num_houses = houses.len();
  let mut hn = 0;
  if (num_houses > 0) {
    let diff = houses[0] / 30f64;
    let hnr = (sign - diff) % num_houses as f64;
    hn = if hnr < 1  { hnr + num_houses } else { hnr };
  }
  return hn;
};

pub fn limit_value_to_range(num: f64, min: f64, max: f64) -> f64 {
  let span = max - min;
  let val = (num - min) % span;
  let ref_val = val > 0 ? val : span + val;
  let out_val = ref_val + min;
  if (min < 0 && (val < 0 || num > max)) { 0 - out_val } else { out_val };
}

pub fn calc_varga_value(lng: f64, num = 1) -> f64 {
  (lng * num as f64) % 360f64
}

pub fn subtract_360(lng: f64, offset = 0) -> f64 {
  (lng + 360 as f64 - offset as f64) % 360f64
}

pub fn calc_all_vargas(lng: f64) -> {
  return VARGA_VALUES.map(|v| {
    let value = calc_varga_value(lng, v.num);
    return { num: v.num, value };
  });
}

pub fn calc_varga_set(lng: f64, num = u8, key: &str) -> {
  let values = calc_all_vargas(lng);
  return {
    num,
    key,
    values,
  };
}

pub fn calc_inclusive_distance(
  pos_1: u16,
  pos_2: u16,
  base: u16,
) -> u16 {
  ((pos_1 - pos_2 + base) % base) + 1
}

pub fn calc_inclusive_twelfths(pos_1: u16, pos_2: u16) -> u16 {
  calc_inclusive_distance(pos_1, pos_2, 12)
}
  

pub fn calc_inclusive_sign_positions(sign1: u8, sign2: u8) -> u8 {
  calc_inclusive_twelfths(sign2 as u16, sign1 as u16, 12) as u8
}

pub fn calc_inclusive_nakshatras(pos_1: u8, pos_2: u8) -> u8 {
  calc_inclusive_twelfths(sign2 as u16, sign1 as u16, 27) as u8
}

pub fn mid_point_to_surface(coord1: GeoPos, coord2: GeoPos) -> GeoPos {
  let c1 = geo_to_radians(coord1);
  let c2 = geo_to_radians(coord2);
  let bx = Math.cos(c2.lat) * Math.cos(c2.lng - c1.lng);
  let by = Math.cos(c2.lat) * Math.sin(c2.lng - c1.lng);
  let mid_lat = Math.atan2(
    Math.sin(c1.lat) + Math.sin(c2.lat),
    Math.sqrt((Math.cos(c1.lat) + bx) * (Math.cos(c1.lat) + bx) + by * by),
  );
  let mid_lng = c1.lng + Math.atan2(by, Math.cos(c1.lat) + bx);
  let mid_alt = (c1.alt + c2.alt) / 2f64;
  GeoPos::new( to_degrees(mid_lat), lng: to_degrees(mid_lng), mid_alt)
}

pub fn to_360(lng: f64) -> f64 {
  if lng >= 0f64 { lng + 180f64 }  else { 180f64 - lng };
}

pub fn from_360(lng: f64) -> f64 {
  if lng > 180f64 { lng - 180 } else { 0 - (180 - lng) };
}

pub fn median_lat(v1: f64, v2: f64) -> f64 {
  let offset = 90f64;
  return (v1 + offset + (v2 + offset)) / 2f64 - offset;
};

pub fn median_lng(v1: f64, v2: f64) -> LngLat {
  let offset = 180;
  let direction = v1;
  let full_circle = offset * 2;
  let lngs = [to_360(v2), to_360(v1)];
  lngs.sort();
  const [d1, d2] = lngs;
  let reverse = Math.abs(d2 - d1) > offset;
  let is_west = reverse ? v1 + v2 > 0 : v1 + v2 < 0;
  let res360 = ((d1 + d2) % full_circle) / 2;
  let res_a = from_360(res360) % offset;
  let res1 = if is_west { res_a } : { 0 - res_a };
  let res_b = offset - res1;
  let res2a = if is_west  { full_circle - res_b } else { res_b };
  let res2 = if is_west && res2a > 0  { 0 - res2a } else { res2a };
  let res2_is_between = to_360(res2) > d1 && to_360(res2) <= d2;
  let result = if reverse || res2_is_between  { res2 } else { res1 };
  return result;
}

pub fn median_lat_lng(coord1: GeoPos, coord2: GeoPos) -> LngLat {
  LngLat::new(
    median_lng(coord1.lng, coord2.lng),
    median_lat(coord1.lat, coord2.lat),
  )
}
