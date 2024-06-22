use std::io;

use crate::{fp::{Vec3F, FP128}, tree::{Body, Cell, CellVisibility}};

use self::fs::{ModError, ModFs};

pub mod fs;

/// approximation of black body spectrum (normalised)
fn black_body(wavelength: f64, temp: f64) -> f64 {
    let peak = 2897771.955 / temp;
    let x_scale = 6.8e-8;
    let y_scale = peak.powi(5) * ((1.0/(peak*x_scale*temp)).exp() - 1.0);
    let denom = wavelength.powi(5) * (1.0/(wavelength*x_scale*temp)).exp() - 1.0;
    return y_scale/denom;
}

/// area of a gaussian with sd (b-a)/2, and mean (b+a)/2
fn gaussian_area(temp: f64, a: f64, b: f64) -> f64 {
    let mid = (a + b) / 2.0;
    let peak = black_body(mid, temp);
    return peak * (b - a) * (2.0 * std::f64::consts::PI).sqrt();
}

/// approximate black body rgb colour given star temperature (multiply with brightness to get luminance)
fn temperature_rgb(temp: f64) -> glam::DVec3 {
    let r = gaussian_area(temp, 520.0, 630.0);
    let g = gaussian_area(temp, 500.0, 590.0);
    let b = gaussian_area(temp, 410.0, 480.0);

    let v = glam::dvec3(r, g, b);

    return v / v.max_element().max(0.00001);
}

/// B-V colour index to temperature
fn ci_temperature(b_v_index: f64) -> f64 {
    let epsilon = 0.001;
    let b_v_index = b_v_index.max(-0.62 / 0.92 + epsilon); // some stars have a colour index lower than this, idk why
    
    let temperature = 4600.0f64*(1.0/(0.92*b_v_index + 1.7) + 1.0/(0.92*b_v_index + 0.62));
    temperature
}

/// absolute magnitude to RGB brightness (arbitrary really, I calibrated it by taking the furthest visible star, and adjusted the value until it was just barely visible)
fn abs_mag_brightness(abs_mag: f64) -> f64 {
    2.512f64.powf(-abs_mag) * 1.0e36
}

fn generate_cell(id: u128, bounds: (Vec3F, Vec3F), luminosity: glam::DVec3) -> Cell {
    println!("generating cell {id}");
    Cell::new(bounds.0, bounds.0, luminosity)
}

pub struct Universe {
    root: Cell,
}

impl Universe {
    // pub const REGION_SIZE: FP128 = fixed_macro::fixed!(1208925819614629174706176: I96F32); // 2^80m, roughly 128 million light years
    pub const REGION_SIZE: FP128 = fixed_macro::fixed!(4951760157141521099596496896: I96F32); // 2^92m, roughly 523 billion light years, 5.63 times the size of the observable universe

    pub fn new() -> Result<Universe, ModError> {
        let colour_index = 3.4;

        let brightness = 2.512f64.powf(-54.0);
        let temperature = 4600.0f64*(1.0/(0.92*colour_index + 1.7) + 1.0/(0.92*colour_index + 0.62));
        let colour = temperature_rgb(temperature) * brightness * 1.0e36;

        let mod_fs = ModFs::new()?;
        
        let mut universe = Universe {
            root: Cell::new(Vec3F::ONE * -Self::REGION_SIZE / 2.0, Vec3F::ONE * Self::REGION_SIZE / 2.0, colour),
        };

        let mut stars = Vec::new();

        log::info!("loading star catalogues");
        for path in mod_fs.read_dir("catalogues/stars")? {
            let catalogue = mod_fs.decompress_bin::<StarCatalogue>(&path)?;
            log::info!("loaded star catalogue {:?} ({} stars)", path.file_name().expect("attempted to open a non-file star catalogue"), catalogue.stars.len());
            stars.extend(catalogue.stars);
        }

        for star in stars {
            let temperature = ci_temperature(star.colour_index);
            let brightness = abs_mag_brightness(star.abs_mag);
            let colour = temperature_rgb(temperature) * brightness;

            // if star.name == "Gacrux" || star.name == "Acrux" || star.name == "Mimosa" || star.name == "Imai" {
            //     colour *= glam::DVec3::Y;
            // }

            universe.root.add_body(Body {
                position: star.pos,
                colour,
            });
        }
        log::info!("placed stars in octree");

        Ok(universe)
    }

    pub fn all_visible_from(&mut self, point: Vec3F, fovy: f32, screen_height: u32) -> Vec<CellVisibility> {
        self.root.all_visible_from(point, fovy, screen_height, &mut generate_cell)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StarCatalogueRecord {
    pub name: String,
    pub pos: Vec3F,
    pub colour_index: f64,
    pub abs_mag: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StarCatalogue {
    pub stars: Vec<StarCatalogueRecord>,
}

impl StarCatalogue {
    pub fn from_csv<T: io::Read>(mut reader: csv::Reader<T>) -> csv::Result<StarCatalogue> {
        #[derive(serde::Deserialize)]
        struct Record {
            name: String,
            x: f64,
            y: f64,
            z: f64,
            colour_index: f64,
            abs_mag: f64,
        }
        
        let mut catalogue = StarCatalogue {
            stars: Vec::new(),
        };
        
        for record in reader.deserialize::<Record>() {
            let Record {
                name,
                x,
                y,
                z,
                colour_index,
                abs_mag,
            } = record?;

            catalogue.stars.push(StarCatalogueRecord {
                name,
                pos: Vec3F::from_dvec3(glam::dvec3(x, y, z) * 3.086e+16), // convert from parsecs to m
                colour_index,
                abs_mag,
            });
        }

        Ok(catalogue)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GalaxyCatalogueRecord {
    pub name: String,
    pub pos: Vec3F,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub diameter: f64,
    pub thickness_stddev: f64,
    pub abs_mag: f64,
    pub colour: String,
    pub height: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GalaxyCatalogue {
    pub galaxies: Vec<GalaxyCatalogueRecord>,
}

impl GalaxyCatalogue {
    pub fn from_csv<T: io::Read>(mut reader: csv::Reader<T>) -> csv::Result<GalaxyCatalogue> {
        #[derive(serde::Deserialize)]
        struct Record {
            name: String,
            x: f64,
            y: f64,
            z: f64,
            nx: f32,
            ny: f32,
            nz: f32,
            tx: f32,
            ty: f32,
            tz: f32,
            diameter: f64,
            thickness_stddev: f64,
            abs_mag: f64,
            colour: String,
            height: String,
        }
        
        let mut catalogue = GalaxyCatalogue {
            galaxies: Vec::new(),
        };
        
        for record in reader.deserialize::<Record>() {
            let Record {
                name,
                x,
                y,
                z,
                nx,
                ny,
                nz,
                tx,
                ty,
                tz,
                diameter,
                thickness_stddev,
                abs_mag,
                colour,
                height,
            } = record?;

            catalogue.galaxies.push(GalaxyCatalogueRecord {
                name,
                pos: Vec3F::from_dvec3(glam::dvec3(x, y, z)),
                normal: glam::vec3(nx, ny, nz),
                tangent: glam::vec3(tx, ty, tz),
                diameter,
                thickness_stddev,
                abs_mag,
                colour,
                height,
            });
        }

        Ok(catalogue)
    }
}
