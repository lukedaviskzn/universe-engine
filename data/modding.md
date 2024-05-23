# Modding Universe Engine

## Mod File System

Mods are stored in `mods` directory, and are loaded in the order given in `loadorder.txt`. Each mod has a `mod.ron` file, which stores the mod's metadata, consisting of the following data:

* name: display name of the mod, e.g. "Core" for the `core` mod, which contains the main engine data files.
* version: mod version (semver.org format), e.g. "0.1.5"
* engine_version: engine version requirement (using VersionReq from Rust `semver` crate), e.g. suppose we are on engine version 1.2.3. "1.2" matches all engine versions 1.2.x, "1" matches all 1.x.y, ">=1.2.3" matches all versions released after version 1.2.3. It is recommended to put just the engine major version number, so just "1" for v1.2.3. (By semver rules as minor versions are supposed to be backwards compatible.)
* author: your name or pseudonym.

When the engine needs a file from a mod (e.g. "catalogues/stars/nearby_stars.bin.gz") it will try to find the file starting at the last item in the load order. Mods not in the load order are ignored.

Note that a mod with an incompatible engine version will be skipped, an error message will be logged to the console.

# Catalogue CSV Format

To see how to encode a catalogue use the command `universe-engine encode-catalogue --help`. You cannot just provide the engine a csv file, you have to encode it into a compressed binary format first.

## Stars

* name
* x, y, z: Cartesian position in parsecs (see below for conversion from RA, dec, dist)
* colour_index: B-V colour index
* abs_mag: absolute magnitude

Most stars sourced from AT-HYG v2.4 (https://www.astronexus.com/hyg).

## Galaxies

* name
* x, y, z: position of centre (parsecs)
* nx, ny, nz: normal vector (normalised, no units)
* tx, ty, tz: tangent vector (normalised, no units)
* diameter: diameter of colour map in real space (parsecs)
* thickness_stddev: standard deviation of the thickness of the galaxy (galaxy thickness approximated as a normalised gaussian curve) (parsecs)
* abs_mag: absolute magnitude
* colour: path to colour picture (true colour unless you want weird looking stars)
* height: path to height map

The engine will fill in the galaxy with stars with the star catalogues above, and will then start procedurally generating stars.

## Nebulae

To do.

## Planets

To do.

# Conversions

## Coordinate System

The origin of the coordinate system is the centre of earth.

X is towards RA 0, Dec 0
Y is towards RA 6hr
Z is towards Dec 90

Converting from RA, Dec, Dist to X, Y, Z can be done as follows:

x = dist * cos(dec) cos(ra)
y = dist * cos(dec) sin(ra)
z = dist * sin(dec)

# Debugging

Run the engine with the environment variable `RUST_LOG=universe-engine=LOG_LEVEL`, where `LOG_LEVEL` is one of the following:
* error (only show errors, default)
* warn (also show warnings)
* info (also other information)
* debug (also show debug information)
* trace (show everything)
