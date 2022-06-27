# astro-calc

This application extends [Stéphane Bressani's](https://github.com/stephaneworkspace/libswe-sys) Rust bridge for Swiss Ephemeris libraries to support:

* Planetary transitions for the referenced day/night period
* Ayanamshas for Indian astrology
* Indian times based on sunrise and sunset
* Transposed transitions of any celestial points
* Altitude calculations
* Easy conversion of Kronos datetimes (NaiveDatetimes) or ISO Datetime strings to and from Julian Days

The main.rs file acts as the API controller for all astronomical and astrological methods and will eventually move to separate files. The key Swiss Epheremris functions requiring unsafe FFI calls that are not supported by the libswe-sys library, are implemented in extensions/swe.ts.

The project is evolving into an open-source API server, using ActixWeb, with a rich set of astronomical and astrological features that may be used by other custom services in creative ways.

## Build instructions:
You may use `cargo build` to build an executable for your operating system (all versions of Linux, Mac or Windows supported by Rust 1.61). However, you will have to configure the Swiss Ephemeris data library. This may already be available if you have installed other versions of Swiss Ephemeris. On Linux libswe is installed at `/usr/share/libswe/ephe`. However, the source files can be downloaded from [www.astro.com/ftp/swisseph/](https://www.astro.com/ftp/swisseph/).

## Commad line parameters

* -e: ephemeris path
* -p: port number

## Endpoints

GET /jd/:datetef

Julian day, unix time stamp and UTC date-time string

Path parameters

* :dateref: either ISO date string with optional time or julian day
  
### GET /positions

Longitudes of referenced celestial bodies and the ascendant. This may power simplified astrological charts. Use this endpoint, if all you need are longitudes, the ascendants plus sun and moon transitions for the day in question.

Query string parameters:

* dt: Date
* loc: lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core
* topo: 0 = geocentric, 1 topocentric
* eq: 0 = ecliptic, 1 equatorial

### GET /progress

Progress of celestial body positionss. This may power charts and 3D animations of planetary orbits over time

Query string parameters:

* dt: start date
* loc: lat,lng(,alt) coordinates, required for topocentric
* bodies: comma-separated list of required bodies, all or core
* days: number of days worth of transitions, default 28, e.g. 366 will return a whole year
* pd: number of samples per day, default 2, i.e. every 12 hours
* dspan: number of days per sample, overrides pd above for longer spans, max 1000 samples
* topo: 0 = geocentric, 1 topocentric
* eq: 0 = ecliptic only, 1 equatorial only

### GET /chart-data

Rich configurable set of astrological data for a given time and geolocation. May power astrological charts with extra transitions and ayanamshas, progress items (P2), transitions, ayanamsha variants and house systems

Query string parameters:

* dt: Date
* loc: lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core
* topo: 0 = geocentric, 1 topocentric
* eq: 0 = ecliptic only, 1 equatorial only
* hsys: Comma-separated list of house system letters or `all` for all systems, default W (whole house system)
* aya: comma-separated list of available ayanamshas (see below). These are added as separate data-set and should be applied in a post processing stage via simple subtraction from the lng, ascendant or rectAscension values, which are always tropical (they may automatically applied in /positions)
* p2: include progress longitudes based on 1 day = 1 year from referenced time. The progress day is mapped to years
* p2yrs: Number of years to capture for P2 data
* p2ago: Number of years ago for P2 start year
* p2start: Explcit start year for p2 data (alternative to above
* p2py: Number of p2 sample per year, default 2.
* p2bodies: Bodies to captured for P2. These never include Uranus, Neptune, Pluto or asteroid. Narrow range to limit the payload

### GET /transitions

* dt: current date-time
* loc: current lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core")

### GET /sun-transitions

Query string parameters:

* dateref: current date-time
* loc: current lat,lng(,alt) coordinates
* days: number of days worth of transitions, default 28, e.g. 366 will return a whole year")

### GET /transposed-transitions

This shows the proejcted transitions of historic celestial body positions transposed onto the current or referenced time and place. It may be used for natal transitions.

Query string parameters:

* dt: current date-time
* loc: current lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core
* dt2: date of source chart
* loc2: coordinates of source chart

## Option Legend

### Celestial Bodies / Planets, Sun, moons, asteroids etc. / Grahas

* su: Sun
* mo: Moon
* me: Mercury
* ve: Venus
* ea: Earth
* ma: Mars
* ju: Jupiter
* sa: Saturn
* ne: Neptune
* ur: Uranus
* pl: Pluto
* ra: True Node / Rahu,
* ke: Opposite True Node / Ketu,
* mn: Mean Node
* sn: South Node
* kr: Kronos
* is: Isis
* jn: Juno
* ce: Ceres
* ch: Chiron

### Ayanamshas (sidereal mode offsets)

* all: All variants listed below
* true_citra: True Citra
* lahiri: Lahiri
* krishnamurti: Krishnamurti
* yukteshwar: Yukteshwar
* raman: Raman
* valensmoon: Valensmoon
* true_mula: True Mula
* true_revati: True Revati
* true_pushya: True Pushya
* true_sheoran: True Sheoran
* aldebaran_15_tau: Aldebaran 15 Tau
* galcent_mula_wilhelm: Galcent Mula Wilhelm
* galcent_cochrane: Galcent Cochrane
* hipparchos: Hipparchos
* sassanian: Sassanian
* ushashashi: Sassanian
* jnbhasin: Jnbhasin
