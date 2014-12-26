use std::num::Float;

/// Vector is a standard 3 component vector
#[deriving(Show, Copy, PartialEq, PartialOrd)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    /// Initialize the vector and set values for x, y, z
    pub fn new(x: f32, y: f32, z: f32) -> Vector {
        Vector { x: x, y: y, z: z }
    }
    /// Initialize the vector with the same value of x, y, z
    pub fn broadcast(x: f32) -> Vector {
        Vector { x: x, y: x, z: x }
    }
    /// Compute the squared length of the vector
    pub fn length_sqr(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    /// Compute the length of the vector
    pub fn length(&self) -> f32 {
        Float::sqrt(self.length_sqr())
    }
    /// Get a normalized copy of this vector
    pub fn normalized(&self) -> Vector {
        let len = self.length();
        Vector { x: self.x / len, y: self.y / len, z: self.z / len }
    }
}

impl Add<Vector, Vector> for Vector {
    /// Add two vectors together
    fn add(self, rhs: Vector) -> Vector {
        Vector { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl Sub<Vector, Vector> for Vector {
    /// Subtract two vectors
    fn sub(self, rhs: Vector) -> Vector {
        Vector { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl Mul<Vector, Vector> for Vector {
    /// Multiply two vectors
    fn mul(self, rhs: Vector) -> Vector {
        Vector { x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z }
    }
}

impl Mul<f32, Vector> for Vector {
    /// Scale the vector by some value
    fn mul(self, rhs: f32) -> Vector {
        Vector { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Div<Vector, Vector> for Vector {
    /// Divide the vectors components by the right hand side's components
    fn div(self, rhs: Vector) -> Vector {
        Vector { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z }
    }
}

impl Div<f32, Vector> for Vector {
    /// Divide the vectors components by a scalar
    fn div(self, rhs: f32) -> Vector {
        Vector { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Neg<Vector> for Vector {
    /// Negate the vector
    fn neg(self) -> Vector {
        Vector { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl Index<uint, f32> for Vector {
    /// Access the vector by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2+ = z
    fn index(&self, i: &uint) -> &f32 {
        match *i {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
}

impl IndexMut<uint, f32> for Vector {
    /// Access the vector by index
    ///
    /// - 0 = x
    /// - 1 = y
    /// - 2+ = z
    fn index_mut(&mut self, i: &uint) -> &mut f32 {
        match *i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => &mut self.z,
        }
    }
}

#[test]
fn test_len_sqr() {
    let v = Vector::new(1f32, 2f32, 3f32);
    assert!(v.length_sqr() == 1f32 + 4f32 + 9f32);
}

#[test]
fn test_idx() {
    let mut v = Vector::new(1f32, 2f32, 3f32);
    assert!(v[0] == 1f32 && v[1] == 2f32 && v[2] == 3f32);
    {
        let x = &mut v[1];
        *x = 5f32;
    }
    assert!(v[1] == 5f32);
}

