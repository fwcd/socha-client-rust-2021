use std::{collections::HashMap, fmt, str::FromStr};
use lazy_static::lazy_static;
use crate::util::{SCResult, SCError, FromXmlNode, XmlNode};
use super::{BOARD_SIZE, Vec2, ROTATIONS, Rotation};

lazy_static! {
    pub static ref PIECE_SHAPES: [PieceShape; 21] = [
        PieceShape::new("MONO", vec![Vec2::new(0, 0)]),
        PieceShape::new("DOMINO", vec![Vec2::new(0, 0), Vec2::new(1, 0)]),
        PieceShape::new("TRIO_L", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(1, 1)]),
        PieceShape::new("TRIO_I", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2)]),
        PieceShape::new("TETRO_O", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(1, 1)]),
        PieceShape::new("TETRO_T", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(2, 0), Vec2::new(1, 1)]),
        PieceShape::new("TETRO_I", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2), Vec2::new(0, 3)]),
        PieceShape::new("TETRO_L", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2), Vec2::new(1, 2)]),
        PieceShape::new("TETRO_Z", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(1, 1), Vec2::new(2, 1)]),
        PieceShape::new("PENTO_L", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2), Vec2::new(0, 3), Vec2::new(1, 3)]),
        PieceShape::new("PENTO_T", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(2, 0), Vec2::new(1, 1), Vec2::new(1, 2)]),
        PieceShape::new("PENTO_V", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2), Vec2::new(1, 2), Vec2::new(2, 2)]),
        PieceShape::new("PENTO_S", vec![Vec2::new(1, 0), Vec2::new(2, 0), Vec2::new(3, 0), Vec2::new(0, 1), Vec2::new(1, 1)]),
        PieceShape::new("PENTO_Z", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(1, 1), Vec2::new(1, 2), Vec2::new(2, 2)]),
        PieceShape::new("PENTO_I", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(0, 2), Vec2::new(0, 3), Vec2::new(0, 4)]),
        PieceShape::new("PENTO_P", vec![Vec2::new(0, 0), Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(0, 2)]),
        PieceShape::new("PENTO_W", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(1, 2), Vec2::new(2, 2)]),
        PieceShape::new("PENTO_U", vec![Vec2::new(0, 0), Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(2, 1), Vec2::new(2, 0)]),
        PieceShape::new("PENTO_R", vec![Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(1, 2), Vec2::new(2, 1), Vec2::new(2, 0)]),
        PieceShape::new("PENTO_X", vec![Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(1, 1), Vec2::new(2, 1), Vec2::new(1, 2)]),
        PieceShape::new("PENTO_Y", vec![Vec2::new(0, 1), Vec2::new(1, 0), Vec2::new(1, 1), Vec2::new(1, 2), Vec2::new(1, 3)])
    ];

    pub static ref PIECE_SHAPES_BY_NAME: HashMap<String, PieceShape> = {
        let mut m = HashMap::new();
        for piece in PIECE_SHAPES.iter() {
            m.insert(piece.name.to_owned(), piece.clone());
        }
        m
    };
}

const MAX_SIDE_LENGTH: i32 = 5;

/// An efficient representation of a piece shape's normalized coordinates.
/// Since every piece shape is less than 5x5 is size, we can represent it
/// using a 5x5 bit-matrix:
///
/// ```text
///  +---+---+---+---+----+
///  | 0 | 1 | 2 | 3 |  4 |
///  +---+---+---+---+----+
///  | 5 | 6 |            |
///  +---+---+    ...     |
///  |                    |
///  +               +----+
///  |               | 24 |
///  +---+---+---+---+----+
/// ```
///
/// These bits are stored in the right-end of of a 32-bit integer.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct CoordinateSet {
    bits: u32
}

impl CoordinateSet {
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    fn index_of(coordinates: Vec2) -> usize {
        assert!(coordinates.x >= 0 && coordinates.y >= 0, "Coordinates have to be positive!");
        assert!(coordinates.y < MAX_SIDE_LENGTH && coordinates.y < MAX_SIDE_LENGTH, "Vec2 are out of bounds!");

        let i = (coordinates.y * MAX_SIDE_LENGTH) + coordinates.x;
        i as usize
    }

    /// Inserts a pair of coordinates (inside the 5x5 box) into the set.
    pub fn insert(&mut self, coordinates: Vec2) {
        self.bits |= 1 << Self::index_of(coordinates);
    }

    /// Checks whether the set contains a given pair of coordinates.
    pub fn contains(&self, coordinates: Vec2) -> bool {
           coordinates.x >= 0
        && coordinates.y >= 0
        && coordinates.x < MAX_SIDE_LENGTH
        && coordinates.y < MAX_SIDE_LENGTH
        && ((self.bits >> Self::index_of(coordinates)) & 1) == 1
    }
}

impl<I> From<I> for CoordinateSet where I: Iterator<Item=Vec2> {
    fn from(coordinates: I) -> Self {
        let mut set = Self::new();

        for coordinates in coordinates {
            set.insert(coordinates);
        }

        set
    }
}

impl fmt::Display for CoordinateSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..MAX_SIDE_LENGTH {
            for x in 0..MAX_SIDE_LENGTH {
                write!(f, "{}", if self.contains(Vec2::new(x, y)) { '#' } else { '.' })?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

struct CoordinateSetIterator {
    bits: u32,
    i: i32
}

impl Iterator for CoordinateSetIterator {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < (MAX_SIDE_LENGTH * MAX_SIDE_LENGTH) {
            let i = self.i;
            let bits = self.bits;

            self.bits >>= 1;
            self.i += 1;

            if (bits & 1) == 1 {
                return Some(Vec2::new(i % MAX_SIDE_LENGTH, i / MAX_SIDE_LENGTH));
            }
        }
        
        None
    }
}

impl IntoIterator for CoordinateSet {
    type Item = Vec2;
    type IntoIter = CoordinateSetIterator;

    fn into_iter(self) -> Self::IntoIter {
        CoordinateSetIterator { bits: self.bits, i: 0 }
    }
}

/// Represents a shape in Blokus. There are 21 different kinds of these.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PieceShape {
    /// The shape's internal name.
    name: &'static str,
    /// The normalized coordinates that make up the shape.
    coordinates: CoordinateSet
}

impl PieceShape {
    fn new(name: &'static str, coordinates: impl IntoIterator<Item=Vec2>) -> Self {
        Self { name, coordinates: CoordinateSet::from(coordinates.into_iter()) }
    }

    /// The piece's (internal) name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Checks whether the piece shape contains the provided (normalized) coordinate pair.
    pub fn contains(&self, coordinates: Vec2) -> bool {
        self.coordinates.contains(coordinates)
    }

    /// A list of occupied fields, with the upper left corner being
    /// the origin (0, 0), the x-axis pointed right and the y-axis pointed downwards
    pub fn coordinates(&self) -> impl Iterator<Item=Vec2> {
        self.coordinates.into_iter()
    }

    /// Prints a human-readable ASCII-art of the coordinates to a string.
    pub fn ascii_art(&self) -> String {
        format!("{}", self.coordinates)
    }

    /// Mirrors this shape by negating all coordinates.
    fn mirror(&self) -> Self {
        Self::new(self.name(), Self::align(self.coordinates().map(|c| -c).collect()))
    }

    /// Turns this piece 90 degrees to the right.
    fn turn_right(&self) -> Self {
        Self::new(self.name(), Self::align(self.coordinates().map(|c| c.turn_right()).collect()))
    }

    /// Turns this piece 90 degrees to the left.
    fn turn_left(&self) -> Self {
        Self::new(self.name(), Self::align(self.coordinates().map(|c| c.turn_left()).collect()))
    }

    /// Flips this piece along the y-axis.
    pub fn flip(&self) -> Self {
        Self::new(self.name(), Self::align(self.coordinates().map(|c| c.flip()).collect()))
    }

    /// Adjusts the coordinates of this piece shape to be relative
    /// to its minimum coords.
    fn align(coordinates: Vec<Vec2>) -> impl Iterator<Item=Vec2> {
        let min_coords = coordinates.iter().fold(Vec2::new(BOARD_SIZE as i32, BOARD_SIZE as i32), |m, &c| m.min(c));
        coordinates.into_iter().map(move |c| c - min_coords)
    }

    /// Performs a rotation of this piece shape.
    pub fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => self.clone(),
            Rotation::Mirror => self.mirror(),
            Rotation::Right => self.turn_right(),
            Rotation::Left => self.turn_left()
        }
    }

    /// Applies the given rotation/flip-combination.
    pub fn transform(&self, rotation: Rotation, flip: bool) -> Self {
        let mut p = self.rotate(rotation);
        if flip {
            p = p.flip();
        }
        p
    }

    /// Fetches the possible rotation/flip-combinations
    pub fn transformations(&self) -> impl Iterator<Item=(Rotation, bool)> {
        ROTATIONS.iter().flat_map(|&r| [true, false].iter().map(move |&f| (r, f)))
    }

    /// Fetches each variant of this shape.
    pub fn variants(&self) -> impl Iterator<Item=PieceShape> {
        let current = self.clone();
        self.transformations().map(move |(r, f)| current.transform(r, f))
    }

    /// Fetches the bounding box of the piece shape, i.e. the smallest rectangle containing it.
    pub fn bounding_box(&self) -> Vec2 {
        let min = self.coordinates.into_iter().fold(Vec2::zero(), |m, c| m.min(c));
        let max = self.coordinates.into_iter().fold(Vec2::zero(), |m, c| m.max(c));
        max - min
    }
}

impl FromStr for PieceShape {
    type Err = SCError;

    fn from_str(raw: &str) -> SCResult<Self> {
        Ok(PIECE_SHAPES_BY_NAME.get(raw).ok_or_else(|| format!("Could not parse shape {}", raw))?.clone())
    }
}

impl fmt::Display for PieceShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromXmlNode for PieceShape {
    fn from_node(node: &XmlNode) -> SCResult<Self> {
        node.content().parse()
    }
}
