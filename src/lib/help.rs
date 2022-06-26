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

  help.insert("jd/{datetef}".to_string(), info_map(
    vec![( 
      "description", "Julian day, unix time stamp and UTC date-time string"),
      ("{dateref}", "either ISO date string with optional time or julian day"),
    ]
  ));
  
  help.insert("positions".to_string(), info_map(
    vec![( "dt", "Date"),
      ("loc", "lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("topo", "0 = geocentric, 1 topocentric"),
      ("eq", "0 = ecliptic only, 1 equatorial only"),
    ]
  ));
  help.insert("chart-data".to_string(), info_map(
    vec![
      ("dt", "Date"),
      ("loc", "lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("topo", "0 = geocentric, 1 topocentric"),
      ("eq", "0 = ecliptic only, 1 equatorial only"),
    ]
  ));
  help.insert("transitions".to_string(), info_map(
    vec![
      ("dt", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core")
    ]
  ));
  help.insert("transposed-transitions".to_string(), info_map(
    vec![
      ("dt", "current date-time"),
      ("loc", "current lat,lng(,alt) coordinates"),
      ("bodies", "comma-separated list of required bodies, all or core"),
      ("dt2", "date of source chart"),
      ("loc2", "coordinates of source chart")
    ]
  ));
  help
}