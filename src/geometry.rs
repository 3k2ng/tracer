use std::{
    any::Any,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div<f32> for Vector {
    type Output = Vector;

    fn div(self, rhs: f32) -> Self::Output {
        Vector {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Vector {
    pub const ZERO: Vector = Vector {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vector { x, y, z }
    }
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn cross(self, rhs: Self) -> Self {
        Vector {
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

pub type Point = Vector;

#[derive(Debug)]
pub struct Ray {
    origin: Point,
    direction: Vector,
}

impl Ray {
    pub fn new(origin: Point, direction: Vector) -> Self {
        Ray { origin, direction }
    }
    pub fn at(&self, t: f32) -> Point {
        self.origin + t * self.direction
    }
}

pub enum Hit {
    Miss,
    Outside(f32),
    Inside(f32),
}

pub trait Renderable {
    fn hit(&self, ray: &Ray) -> Hit;
    fn normal(&self, ray: &Ray, hit: Hit) -> Option<Vector>;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f32,
}

impl Renderable for Sphere {
    fn hit(&self, ray: &Ray) -> Hit {
        let l = self.center - ray.origin;
        let tca = l.dot(ray.direction);
        let d2 = l.length_square() - tca * tca;
        let r2 = self.radius * self.radius;
        if d2 > r2 {
            Hit::Miss
        } else {
            let thc = (r2 - d2).sqrt();
            let t0 = tca - thc;
            let t1 = tca + thc;
            if t0 > 0. {
                Hit::Outside(t0)
            } else if t1 > 0. {
                Hit::Inside(t1)
            } else {
                Hit::Miss
            }
        }
    }

    fn normal(&self, ray: &Ray, hit: Hit) -> Option<Vector> {
        match hit {
            Hit::Miss => None,
            Hit::Outside(t) => Some((ray.at(t) - self.center).normalize()),
            Hit::Inside(t) => Some((self.center - ray.at(t)).normalize()),
        }
    }
}
