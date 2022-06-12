
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

pub fn map_sign_to_house = (sign: f64, houses: Vec<f64>): f64 => {
  let num_houses = houses.len();
  let mut hn = 0;
  if (num_houses > 0) {
    let diff = houses[0] / 30f64;
    let hnr = (sign - diff) % num_houses as f64;
    hn = if hnr < 1  { hnr + numH } else { hnr };
  }
  return hn;
};

pub fn limitValueToRange = (num = 0, min = 0, max = 360): f64 => {
  let span = max - min;
  let val = (num - min) % span;
  let refVal = val > 0 ? val : span + val;
  let outVal = refVal + min;
  return (min < 0 && (val < 0 || num > max))? 0 - outVal: outVal;
}

pub fn calcVargaValue = (lng: f64, num = 1) => (lng * num) % 360;

pub fn subtract_360(lng: f64, offset = 0) -> f64 {
  (lng + 360 as f64 - offset as f64) % 360f64
}

pub fn calcAllVargas = (lng: f64) => {
  return vargaValues.map(v => {
    let value = calcVargaValue(lng, v.num);
    return { num: v.num, key: v.key, value };
  });
};

pub fn calcVargaSet = (lng, num, key) => {
  let values = calcAllVargas(lng);
  return {
    num,
    key,
    values,
  };
};

pub fn calcInclusiveDistance = (
  posOne: f64,
  posTwo: f64,
  base: f64,
) => ((posOne - posTwo + base) % base) + 1;

pub fn calcInclusiveTwelfths = (posOne: f64, posTwo: f64) =>
  calcInclusiveDistance(posOne, posTwo, 12);

pub fn calcInclusiveSignPositions = (sign1: f64, sign2: f64) =>
  calcInclusiveDistance(sign2, sign1, 12);

pub fn calcInclusiveNakshatras = (posOne: f64, posTwo: f64) =>
  calcInclusiveDistance(posOne, posTwo, 27);

pub fn midPointSurface = (coord1: GeoPos, coord2: GeoPos) => {
  let c1 = geoToRadians(coord1);
  let c2 = geoToRadians(coord2);
  let bx = Math.cos(c2.lat) * Math.cos(c2.lng - c1.lng);
  let by = Math.cos(c2.lat) * Math.sin(c2.lng - c1.lng);
  let midLat = Math.atan2(
    Math.sin(c1.lat) + Math.sin(c2.lat),
    Math.sqrt((Math.cos(c1.lat) + bx) * (Math.cos(c1.lat) + bx) + by * by),
  );
  let midLng = c1.lng + Math.atan2(by, Math.cos(c1.lat) + bx);
  return { lat: toDegrees(midLat), lng: toDegrees(midLng) };
};

pub fn approxTransitTimes = (geo: GeoPos, alt: f64, jd: f64, ra: f64, decl: f64): TransitJdSet => {
  let deltaT = getDeltaT(jd);
  let nut = nutation(jd + deltaT)[0];
  let sidTime = getSidTime(jd, 0, nut);
  let h0 = toRadians(alt);
  const α = toRadians(ra);
  const δ = toRadians(decl);
  //let th0 = sidereal.apparent0UT(jd);
  let th0 = sidTime;
  //let th1 = sidereal.apparent0UT(jd - 0.5);
  let th1 = getSidTime(jd - 0.5, 0, nut);
  let transData = rise.approxTimes({lat: toRadians(geo.lat), lon: toRadians(geo.lng)}, h0, th0, α, δ, th1);
  let result = { rise: 0, set: 0, mc: 0, ic: 0 };
  if (transData instanceof Object) {
    let keys = Object.keys(transData);
    if (keys.includes("rise") && keys.includes("set")) {
      result.rise = secsToExactJd(jd, transData.rise, geo.lng);
      result.set = secsToExactJd(jd, transData.set, geo.lng);
      result.mc = secsToExactJd(jd, transData.mc, geo.lng),
      result.ic = secsToExactJd(jd, transData.ic, geo.lng);
    }
  }
  return result;
}

pub fn to_360 = lng => (lng >= 0 ? lng + 180 : 180 + lng);

pub fn from_360 = lng => (lng > 180 ? lng - 180 : 0 - (180 - lng));

let median_lat = (v1: f64, v2: f64) => {
  let offset = 90;
  return (v1 + offset + (v2 + offset)) / 2 - offset;
};

fn median_lng(v1: f64, v2: f64) -> LngLat {
  let offset = 180;
  let direction = v1;
  let fullC = offset * 2;
  let lngs = [to_360(v2), to_360(v1)];
  lngs.sort();
  const [d1, d2] = lngs;
  let reverse = Math.abs(d2 - d1) > offset;
  let isWest = reverse ? v1 + v2 > 0 : v1 + v2 < 0;
  let res360 = ((d1 + d2) % fullC) / 2;
  let resA = from_360(res360) % offset;
  let res1 = isWest ? resA : 0 - resA;
  let resB = offset - res1;
  let res2a = isWest ? fullC - resB : resB;
  let res2 = isWest && res2a > 0 ? 0 - res2a : res2a;
  let res2isBetween = to_360(res2) > d1 && to_360(res2) <= d2;
  let result = reverse || res2isBetween ? res2 : res1;
  return result;
}

pub fn median_lat_lng(coord1: GeoPos, coord2: GeoPos) -> LngLat {
  LngLat::new(
    median_lng(coord1.lng, coord2.lng),
    median_lat(coord1.lat, coord2.lat),
  )
}
