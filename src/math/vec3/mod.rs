use crate::traits::Vector;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
///A generic vector with 3 dimensions
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Vector for Vec3 {
    fn length(&self) -> f32 {
        self.square_length().sqrt()
    }

    fn square_length(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn dot_product(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl Into<Vec3> for (f32, f32, f32) {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}
// (x,0,0
//  0,y,0,
//  0,0,z)
//  (1,0,0,x,
//   0,1,0,y,
//   0,0,1,z
//   0,0,0,1)
//

#[test]
fn test_vec3_dot_product() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    let expected = 0.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(1.0, 0.0, 0.0);

    let expected = 1.0;
    let result = a.dot_product(&b);

    assert_eq!(expected, result);
}

#[test]
fn test_vec3_length() {
    let a = Vec3::new(1.0, 2.0, 2.0);
    assert_eq!(a.square_length(), 9.0);
    assert_eq!(a.length(), 3.0);
}
