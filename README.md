# astro-calc

This application extends [Stéphane Bressani's](https://github.com/stephaneworkspace/libswe-sys) Rust bridge for Swiss Ephemeris libraries to support:

* Planetary transitions for the referenced day/night period
* Ayanamshas for Indian astrology
* Indian times based on sunrise and sunset
* Transposed transitions of any celestial points
* Altitude calculations
* Easy conversion of Kronos datetimes (NaiveDatetimes) or ISO Datetime strings to and from Julian Days

It will eventually become an open-source API server, using ActixWeb, with a rich set of astrological features.