use std::{fmt::Debug, hash::Hash};

use fixed::traits::ToFixed;

use crate::fp::{Vec3F, FP128};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub position: Vec3F,
    pub colour: glam::DVec3,
}

impl Body {
    fn position(&self) -> Vec3F {
        self.position
    }
    
    fn diameter(&self) -> FP128 {
        1.0.to_fixed()
    }

    fn luminosity(&self) -> glam::DVec3 {
        self.colour
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // SAFETY: do not edit octant without changing conversion in Sector::tree_coord().
pub enum Octant {
    NxNyNz,
    NxNyPz,
    NxPyNz,
    NxPyPz,
    PxNyNz,
    PxNyPz,
    PxPyNz,
    PxPyPz,
}

impl Octant {
    pub const ALL: [Self; 8] = [
        Self::NxNyNz,
        Self::NxNyPz,
        Self::NxPyNz,
        Self::NxPyPz,
        Self::PxNyNz,
        Self::PxNyPz,
        Self::PxPyNz,
        Self::PxPyPz,
    ];
}

impl From<Octant> for Vec3F {
    fn from(octant: Octant) -> Self {
        let one = fixed!(1.0: I96F32);
        let zero = fixed!(0.0: I96F32);
        match octant {
            Octant::NxNyNz => Vec3F::new(zero, zero, zero),
            Octant::NxNyPz => Vec3F::new(zero, zero, one),
            Octant::NxPyNz => Vec3F::new(zero, one, zero),
            Octant::NxPyPz => Vec3F::new(zero, one, one),
            Octant::PxNyNz => Vec3F::new(one, zero, zero),
            Octant::PxNyPz => Vec3F::new(one, zero, one),
            Octant::PxPyNz => Vec3F::new(one, one, zero),
            Octant::PxPyPz => Vec3F::new(one, one, one),
        }
    }
}

impl From<(bool, bool, bool)> for Octant {
    fn from((x, y, z): (bool, bool, bool)) -> Self {
        use Octant::*;
        if x {
            if y {
                if z { PxPyPz }
                else { PxPyNz }
            } else {
                if z { PxNyPz }
                else { PxNyNz }
            }
        } else {
            if y {
                if z { NxPyPz }
                else { NxPyNz }
            } else {
                if z { NxNyPz }
                else { NxNyNz }
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sector {
    id: u128,
    /// (min, max)
    bounds: (Vec3F, Vec3F),
    centre: Vec3F,
    luminosity: glam::DVec3,
    depth: usize,
}

impl Sector {
    #[allow(unused)]
    fn new(id: u128, bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3) -> Self {
        Self::with_depth(id, bound_min, bound_max, luminosity, 0)
    }
    
    fn with_depth(id: u128, bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3, depth: usize) -> Self {
        Self {
            id,
            bounds: (bound_min, bound_max),
            centre: (bound_min + bound_max) / 2.0,
            luminosity,
            depth,
        }
    }

    fn octant(&self, point: Vec3F) -> Option<Octant> {
        let (min, max) = self.bounds;
        if point.x < min.x || point.y < min.y || point.z < min.z || max.x <= point.x || max.y <= point.y || max.z <= point.z {
            return None;
        }

        Some(Octant::from((point.x >= self.centre.x, point.y >= self.centre.y, point.z >= self.centre.z)))
    }

    fn dimensions(&self) -> Vec3F {
        self.bounds.1 - self.bounds.0
    }

    pub fn id(&self) -> u128 {
        self.id
    }

    pub fn luminosity(&self) -> glam::DVec3 {
        self.luminosity
    }

    const ID_ROOT: u128 = 0b111;

    fn calc_id(tree_coord: &[Octant]) -> u128 {
        // all id's start with 111 (7) as a marker
        let mut out = Self::ID_ROOT;
        // [0,1,2] => 0b111_002_001_000
        for c in tree_coord {
            out *= 8;
            out += *c as u128;
        }
        out
    }

    fn tree_coord(mut id: u128) -> Vec<Octant> {
        let mut out = Vec::with_capacity(Cell::MAX_DEPTH);
        while id > Self::ID_ROOT {
            // SAFETY: octant will always have no more than 8 members
            // out.push(unsafe { std::mem::transmute((id % 8) as u8) });
            out.push(Octant::ALL[(id % 8) as usize]);
            id /= 8;
        }
        out.reverse();
        out
    }

    fn id_push(id: u128, oct: Octant) -> u128 {
        id * 8 + oct as u128
    }
}
// u128 capable of holding cell id's
const_assert!(Cell::MAX_DEPTH*3 < 128);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum Node {
    Cell(Box<Cell>),
    Leaf(Leaf),
    Unloaded(u128),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Leaf {
    sector: Sector,
    children: Vec<Body>,
}

impl Leaf {
    #[allow(unused)]
    fn new(bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3) -> Self {
        Self::with_depth(bound_min, bound_max, luminosity, 0, Sector::ID_ROOT)
    }
    
    fn with_depth(bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3, depth: usize, id: u128) -> Self {
        Self {
            sector: Sector::with_depth(id, bound_min, bound_max, luminosity, depth),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub position: Vec3F,
    pub diameter: FP128,
    pub colour: glam::DVec3,
    pub is_body: bool,
}

impl PartialEq for PointLight {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.diameter == other.diameter &&
        self.colour.x as u128 == other.colour.x as u128 &&
        self.colour.y as u128 == other.colour.y as u128 &&
        self.colour.z as u128 == other.colour.z as u128
    }
}

impl Eq for PointLight {}

impl Hash for PointLight {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.diameter.hash(state);
        (self.colour.x as u128).hash(state);
        (self.colour.y as u128).hash(state);
        (self.colour.z as u128).hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellVisibility {
    pub centre: Vec3F,
    pub depth: usize,
    pub bodies: Vec<PointLight>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Cell {
    sector: Sector,
    children: [Node; 8],
}

impl Cell {
    pub const MAX_DEPTH: usize = 40; // 2^92m / 2^40 = 2^52m ~= 0.48ly

    /// inclusive min, exclusive max
    pub fn new(bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3) -> Self {
        Self::with_depth(bound_min, bound_max, luminosity, 0, Sector::ID_ROOT)
    }

    fn with_depth(bound_min: Vec3F, bound_max: Vec3F, luminosity: glam::DVec3, depth: usize, id: u128) -> Self {
        assert!(bound_min.x <= bound_max.x && bound_min.y <= bound_max.y && bound_min.z <= bound_max.z, "invalid cell bounds {bound_min:?} {bound_max:?}");
        
        let centre = (bound_min + bound_max) / 2.0;
        let half = centre - bound_min;
        
        let children = Octant::ALL.map(|o| {
            let min = Vec3F::from(o) * half + bound_min;
            Node::Leaf(Leaf::with_depth(min, min + half, luminosity, depth + 1, Sector::id_push(id, o)))
        });

        Self {
            sector: Sector::with_depth(id, bound_min, bound_max, luminosity, depth),
            children,
        }
    }

    pub fn sector(&self) -> &Sector {
        &self.sector
    }

    /// add body to this cell, panics if body not in bounds
    pub fn add_body(&mut self, body: Body) {
        let pos = body.position();

        let octant = self.sector.octant(pos).expect("point not in cell bounds");

        self.sector.luminosity += body.luminosity();

        match &mut self.children[octant as usize] {
            Node::Cell(cell) => cell.add_body(body),
            Node::Leaf(Leaf { sector, children }) => if children.len() > 0 && sector.depth < Self::MAX_DEPTH {
                self.subdivide(octant);
                let Node::Cell(cell) = &mut self.children[octant as usize] else { unreachable!() };
                cell.add_body(body);
            } else {
                self.sector.luminosity += body.luminosity();
                children.push(body);
            }
            // Node::Unloaded(_) => {
            //     self.subdivide(octant);
            //     let Node::Cell(cell) = &mut self.children[octant as usize] else { unreachable!() };
            //     cell.add_body(body);
            // }
            Node::Unloaded(_) => unreachable!("tried to add body to unloaded cell"),
        }
    }

    fn attenuation(dist: f64, radius: f64) -> f64 {
        let r = dist / radius;
        let att = 1.0 + r;
        
        att * att
    }

    pub fn visible_from(&self, point: Vec3F, fovy_factor: f32) -> bool {
        let dist = Into::<glam::DVec3>::into(self.sector.centre - point).length() - self.sector.dimensions().max().to_num::<f64>();

        // nearby or within cell
        if dist <= 0.0 {
            // is there anything to see?
            return self.sector.luminosity.max_element() > 0.0;
        }

        let att = Self::attenuation(dist, 1.0) * fovy_factor as f64;

        // so far away, that it's not even a pixel in width
        (self.sector.luminosity.max_element() / att) > Self::MIN_BRIGHTNESS
    }

    // this value is purposefully extremely small, we want the leaf nodes to show even if there is only the slightest chance they will be visible,
    // especially given that point lights use additive blending, they may still be visible if overlapping
    const MIN_BRIGHTNESS: f64 = 0.01 / 255.0; // brightness below which not visible
    const MESH_COMBINE_THRESHOLD: usize = 8192;

    pub fn all_visible_from<F: Fn(u128, (Vec3F, Vec3F), glam::DVec3) -> Cell>(&mut self, point: Vec3F, fovy_factor: f32, generate_cell: &mut F) -> Vec<CellVisibility> {
        let mut points = vec![];
        let mut visibility = vec![];
        
        // not visible, neither will children be visible
        if !self.visible_from(point, fovy_factor) {
            return visibility;
        }

        for octant in Octant::ALL {
            let child = &mut self.children[octant as usize];
            match child {
                Node::Cell(child) => {
                    let child_visibility = child.all_visible_from(point, fovy_factor, generate_cell);
                    // combine small cells into larger ones
                    if child_visibility.iter().map(|c| c.bodies.len()).sum::<usize>() < Self::MESH_COMBINE_THRESHOLD {
                        points.extend(child_visibility.into_iter().map(|c| c.bodies).flatten());
                    } else {
                        visibility.extend(child_visibility);
                    }
                },
                Node::Leaf(leaf) => {
                    for child in &leaf.children {
                        let dist = Into::<glam::DVec3>::into(child.position() - point).length();
                        let att = Self::attenuation(dist, 1.0);

                        // child is visible
                        if child.luminosity().max_element() / att > Self::MIN_BRIGHTNESS {
                            points.push(PointLight {
                                position: child.position(),
                                diameter: child.diameter(),
                                colour: child.luminosity(),
                                is_body: true,
                            });
                        }
                    }
                },
                Node::Unloaded(id) => {
                    let cell = {
                        let half = self.sector.centre - self.sector.bounds.0;
                        let min = self.sector.bounds.0 + Vec3F::from(octant) * half;
                        let max = min + half;
                        generate_cell(*id, (min, max), self.sector.luminosity / 8.0)
                    };
                    *child = Node::Cell(Box::new(cell));
                },
            }
        }

        // some children are visible, return children
        if points.len() > 0 || visibility.len() > 0 {
            visibility.push(CellVisibility {
                centre: self.sector.centre,
                depth: self.sector.depth,
                bodies: points,
            });
            
            return visibility;
        }
        
        // no children are visible, return point light approximation
        
        let diameter = self.sector.dimensions().max();
        
        visibility.push(CellVisibility {
            centre: self.sector.centre,
            depth: self.sector.depth,
            bodies: vec![PointLight {
                position: self.sector.centre,
                diameter,
                colour: self.sector.luminosity,
                is_body: false,
            }],
        });
        
        visibility
    }

    fn subdivide(&mut self, octant: Octant) {
        if self.sector.depth >= Self::MAX_DEPTH { return; } // too deep, cannot subdivide

        let Node::Leaf(leaf) = &mut self.children[octant as usize] else { return; }; // already subdivided
        let bodies = leaf.children.drain(..).collect::<Vec<_>>();
        
        let half = self.sector.centre - self.sector.bounds.0;
        let min = self.sector.bounds.0 + Vec3F::from(octant) * half;
        let max = min + half;

        self.children[octant as usize] = Node::Cell(Box::new(Cell::with_depth(min, max, glam::DVec3::ZERO, self.sector.depth + 1, Sector::id_push(self.sector.id, octant))));
        let Node::Cell(cell) = &mut self.children[octant as usize] else { unreachable!() };

        for body in bodies {
            cell.add_body(body);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        // check that we can insert without panicking
        let mut cell = Cell::new(Vec3F::ZERO, Vec3F::ONE, glam::DVec3::ZERO);
        cell.add_body(Body { position: Vec3F::ONE / 5.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 4.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 3.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 2.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 1.8, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 1.6, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 1.4, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE / 1.2, colour: glam::DVec3::ONE });

        let mut cell = Cell::new(Vec3F::ONE, Vec3F::ONE * 2.0, glam::DVec3::ZERO);
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 5.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 4.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 3.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 2.0, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 1.8, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 1.6, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 1.4, colour: glam::DVec3::ONE });
        cell.add_body(Body { position: Vec3F::ONE + Vec3F::ONE / 1.2, colour: glam::DVec3::ONE });
    }

    #[test]
    fn sector_ids() {
        let mut id = Sector::ID_ROOT;
        let mut octs = vec![];
        for o in Octant::ALL {
            octs.push(o);
            id = Sector::id_push(id, o);
            assert_eq!(Sector::tree_coord(id), octs);
        }
        for o in Octant::ALL.iter().rev() {
            octs.push(*o);
            id = Sector::id_push(id, *o);
            assert_eq!(Sector::tree_coord(id), octs);
        }
    }
}
