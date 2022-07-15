# astro-calc

This application extends [Stéphane Bressani's](https://github.com/stephaneworkspace/libswe-sys) Rust bridge for Swiss Ephemeris libraries to support:

* Planetary transitions for the referenced day/night period
* Ayanamshas for Indian astrology
* Indian times based on sunrise and sunset
* Transposed transitions of any celestial points
* Altitude calculations
* Easy conversion of chrono datetimes (NaiveDatetimes) or ISO Datetime strings to and from Julian Days

The main.rs file acts as the API controller for all astronomical and astrological methods and will eventually move to separate files. The key Swiss Epheremris functions requiring unsafe FFI calls that are not supported by the libswe-sys library, are implemented in extensions/swe.ts.

The project is evolving into an open-source API server, using ActixWeb, with a rich set of astronomical and astrological features that may be used by other custom services in creative ways.

## Build instructions:
You may use `cargo build` to build an executable for your operating system (all versions of Linux, Mac or Windows supported by Rust 1.61). However, you will have to configure the Swiss Ephemeris data library. This may already be available if you have installed other versions of Swiss Ephemeris. On Linux libswe is installed at `/usr/share/libswe/ephe`. However, the source files can be downloaded from [www.astro.com/ftp/swisseph/](https://www.astro.com/ftp/swisseph/).

The API is publicly available at [astroapi.findingyou.co](https://astroapi.findingyou.co). This is a sample data-set with [equatorial and ecliptic coordinates as well as transitions of the sun, moon and core planets](https://astroapi.findingyou.co/chart-data?dt=2022-06-01T00:00:00&loc=48.15,6.667&ct=1&topo=1&eq=3&iso=1)

## Commad line parameters

* -e: ephemeris path
* -p: port number

## Endpoints

GET /jd/:datetef

Julian day, unix time stamp and UTC date-time string

Path parameters

* :dateref: either ISO date string with optional time or julian day

GET /date

Show data variants including Indian dates from sunrise to sunrise and detailed sun transition infor based on with query string parameters

* dt: Date
* loc: lat,lng(,alt) coordinates
* iso: 0 = julian days, 1 ISO UTC

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
* iso: 0 = julian days (transition times), 1 ISO UTC

### GET /progress

Progress of celestial body positions. This may power charts and 3D animations of planetary orbits over time

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

Rich configurable set of astrological data for a given time and geolocation. May power astrological charts with extra transitions and ayanamsha variants, progress synastry positions (P2) and house systems. Previous and next planet stations (retrograde motion switches) will be shown if eq is 3 or 4 or ph is 1.

Query string parameters:

* dt: Date
* loc: lat,lng(,alt) coordinates
* bodies: comma-separated list of 2-letter abbreviations for required bodies, all or core
* topo: 0 = geocentric, 1 topocentric
* eq: 
  * 0 = ecliptic only,
  * 1 equatorial only,
  * 2 both ecliptic and equatorial,
  * 3 both with altitude, azimuth and extra planetary phenomena such as magnitude and phase angle. The azimuth and altitude will only be shown in topocentric mode.
  * 4 With extra planetary phenomena such as magnitude and phase angle as an inline subset.
* ph: 0 = no extra phenomena unless eq == 4, 1 = show planetary phenomena for the referenced time unless it is shown inline with celestial body data.
* hsys: Comma-separated list of house system letters or `all` for all systems, default W (whole house system)
* aya: Comma-separated list of available ayanamshas (see below). These are added as separate data-set and should be applied in a post processing stage via simple subtraction from the lng, ascendant or rectAscension values, which are always tropical (they may automatically applied in /positions)
* p2: include progress synastry longitudes based on 1 day = 1 year from referenced time. Progress days since the historic chart data is mapped to years.
* p2yrs: Number of years to capture for P2 data
* p2ago: Number of years ago for the P2 start year
* p2start: Explcit start year for progress synastry data (alternative to above
* p2py: Number of p2 sample per year, default 2.
* p2bodies: Bodies to captured for P2. These never include Uranus, Neptune, Pluto or asteroid. Narrow range to limit the payload

### GET /transitions

* dt: current date-time
* loc: current lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core")
* iso: 0 = show julian days (default), 1 = show ISO datetime UTC

### GET /sun-transitions

Query string parameters:

* dt: current date-time
* loc: current lat,lng(,alt) coordinates
* days: number of days worth of transitions, default 28, e.g. 366 will return a whole year")
* iso: 0 = show julian days (default), 1 = show ISO datetime UTC

### GET /planet-stations

Show retrograde start, retrograde peak, retrograde end and forward peak speeds of the core planets over a specified period:

* dt: start date-time or year only, between 1900 and 2050
* dt2: end date-time or year only, between 1900 and 2050
* bodies: comma-separated list of required planets, all or core, but may only include me: Mercury, ve: Venus, ma: Mars, ju: Jupiter, sa: Saturn, ur: Uranus, ne: Neptune and pl: Pluto
* iso: 0 = show julian days (default), 1 = show ISO datetime UTC

### GET /transposed-transitions

This shows the proejcted transitions of historic celestial body positions transposed onto the current or referenced time and place. It may be used for natal transitions.

Query string parameters:

* dt: referenced date-time
* loc: current lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core
* dt2: date of source chart
* loc2: coordinates of source chart
* iso: 0 = show julian days (default), 1 = show ISO datetime UTC

### GET /test-transitions

Compare transition calculation methods. One uses swe_rise_calc and the other, better suited to polar latitudes uses swe_azalt to approximate transitions by variations in altitude. Eventually, the latter method will be uses for all latitudes > 60º or < -60º.

Query string parameters:

* dt: referenced date-time
* loc: current lat,lng(,alt) coordinates
* bodies: comma-separated list of required bodies, all or core
* iso: 0 = show julian days (default), 1 = show ISO datetime UTC

### GET /pheno

This shows planetary phenomena for the referenced time and celestial bodies. This only applies to visible planets, moons and stars

Query string parameters:

* dt: referenced date-time
* bodies: comma-separated list of required bodies, all or core

## Option Legend

### Celestial Bodies / Planets, Sun, moons, asteroids etc. / Grahas

* all: All planets from Mercury to Pluto (except Earth) + Sun, Moon, Rahu (True Node) and Ketu 
* core: All used in traditional astrology, Sun, Moon, Mars, Mercury, Jupiter, Saturn, Rahu and Ketu 
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

### House Systems

* A: equal
* B: Alcabitius
* C: Campanus
* D: equal (MC)
* E: equal
* F: Carter poli-equ.
* G: Gauquelin sectors
* H: horizon/azimut
* I: Sunshine
* i: Sunshine/alt.
* K: Koch
* L: Pullen SD
* M: Morinus
* N: equal/1=Aries
* O: Porphyry
* Q: Pullen SR
* R: Regiomontanus
* S: Sripati
* T: Polich/Page
* U: Krusinski-Pisa-Goelzer
* V: equal/Vehlow
* W: equal/ whole sign
* X: axial rotation system/Meridian houses
* Y: APC houses

### Ayanamshas (sidereal mode offsets)

* all: All variants listed below
* tc, true_citra: True Citra
* lh, lahiri: Lahiri
* kr, krishnamurti: Krishnamurti
* yu, yukteshwar: Yukteshwar
* ra, raman: Raman
* va, valensmoon: Valensmoon
* tm, true_mula: True Mula
* tr, true_revati: True Revati
* tp, true_pushya: True Pushya
* ts, true_sheoran: True Sheoran
* at, aldebaran_15_tau: Aldebaran 15 Tau
* gm, galcent_mula_wilhelm: Galcent Mula Wilhelm
* gc, galcent_cochrane: Galcent Cochrane
* hi, hipparchos: Hipparchos
* sa, sassanian: Sassanian
* us, ushashashi: Sassanian
* jb, jnbhasin: Jnbhasin
