use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn cross(self, rhs: Self) -> Self {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
    pub fn length_square(self) -> f32 {
        self.dot(self)
    }
    pub fn length(self) -> f32 {
        self.length_square().sqrt()
    }
    pub fn normalize(self) -> Self {
        let length = self.length();
        if length > 0. {
            self / length
        } else {
            self
        }
    }
}

pub type Point3 = Vec3;

#[derive(Debug)]
pub struct Ray {
    origin: Point3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }
    pub fn at(&self, t: f32) -> Point3 {
        self.origin + t * self.direction
    }
}

pub trait Renderable {
    fn hit(&self, ray: &Ray) -> Option<f32>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
}

impl Renderable for Sphere {
    fn hit(&self, ray: &Ray) -> Option<f32> {
        let l = self.center - ray.origin;
        let tca = l.dot(ray.direction);
        let d2 = l.length_square() - tca * tca;
        if d2 > self.radius * self.radius {
            None
        } else {
            let thc = (self.radius * self.radius - d2).sqrt();
            let t0 = tca - thc;
            let t1 = tca + thc;
            let t_min = if t0 > t1 { t1 } else { t0 };
            let t_max = if t0 > t1 { t0 } else { t1 };
            if t_min > 0. {
                Some(t_min)
            } else if t_max > 0. {
                Some(t_max)
            } else {
                None
            }
        }
    }
}
