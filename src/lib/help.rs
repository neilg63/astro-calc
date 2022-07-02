use std::collections::HashMap;

fn info_map(pairs: Vec<(&str, &str)>) -> HashMap<String, String> {
  let mut info: HashMap<String, String> = HashMap::new();
  for pair in pairs {
    info.insert(pair.0.to_owned(), pair.1.to_owned());
  }
  info
}

pub fn endpoint_help() -> HashMap<String, HashMap<String,String>> {
  let mut help: HashMap<String, HashMap<String, String>> = HashMap::new();

  help.insert("/jd/:datetef".to_string(), info_map(
    vec![( 
      "description", "Julian day, unix time stamp and UTC date-time string"),
      (":dateref", "either ISO date string with optional time or julian day"),
    ]
  ));
  
  help.insert("/positions".to_string(), info_map(
    vec![
      ("description", "Longitudes of referenced celestial bodies and the ascendant"),
      ( "dt", "Date"),
      ("loc", "lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("topo", "0 = geocentric, 1 topocentric"),
      ("eq", "0 = ecliptic, 1 equatorial"),
    ]
  ));
  help.insert("/progress".to_string(), info_map(
    vec![
      ("description", "Progress of celestial body positions"),
      ( "dt", "start date"),
      ("loc", "lat,lng(,alt) coordinates, required for topocentric, e.g. &loc=45.336,13.278,50 or just &loc=45.336,13.278"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("days", "number of days worth of transitions, default 28, e.g. 366 will return a whole year"),
      ("pd", "number of samples per day, default 2, i.e. every 12 hours"),
      ("dspan", "number of days per sample, overrides pd above for longer spans, max 1000 samples"),
      ("topo", "0 = geocentric, 1 topocentric"),
      ("eq", "0 = ecliptic only, 1 equatorial only"),
    ]
  ));
  help.insert("/chart-data".to_string(), info_map(
    vec![
      ("dt", "Date"),
      ("loc", "lat,lng(,alt) coordinates, e.g. &loc=45.336,13.278,50 or just &loc=45.336,13.278"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("topo", "0 = geocentric, 1 topocentric"),
      ("eq", "0 = ecliptic only, 1 equatorial only"),
      ("ct", "include transits for the referenced bodies"),
      ("hsys", "Comma-separated list of house system letters or `all` for all systems, default W (whole house system)"),
      ("aya", "comma-separated list of available ayanamshas (see below). These are added as separate data-set and should be applied in a post processing stage via simple subtraction from the lng, ascendant or rectAscension values, which are always tropical (they may automatically applied in /positions)"),
      ("p2", "include progress longitudes based on 1 day = 1 year from referenced time. The progress day is mapped to years"),
      ("p2yrs", "Number of years to capture for P2 data"),
      ("p2ago", "Number of years ago for P2 start year"),
      ("p2start", "Explcit start year for p2 data (alternative to above"),
      ("p2py", "Number of p2 sample per year, default 2."),
      ("p2bodies", "Bodies to captured for P2. These never include Uranus, Neptune, Pluto or asteroid. Narrow range to limit the payload"),
    ]
  ));
  help.insert("/transitions".to_string(), info_map(
    vec![
      ("dt", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core")
    ]
  ));
  help.insert("/sun-transitions".to_string(), info_map(
    vec![
      ("dateref", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("days", "number of days worth of transitions, default 28, e.g. 366 will return a whole year"),
      ("iso", "0 = show julian days (default), 1 = show ISO datetime UTC")
    ]
  ));
  help.insert("/transposed-transitions".to_string(), info_map(
    vec![
      ("dt", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("dt2", "date of source chart"),
      ("loc2", "coordinates of source chart"),
      ("iso", "0 = show julian days (default), 1 = show ISO datetime UTC"),
    ]
  ));

  help.insert("/pheno".to_string(), info_map(
    vec![
      ("dt", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
    ]
  ));
  help
}