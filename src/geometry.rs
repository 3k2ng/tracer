use std::ops::{Add, Div, Mul, Sub};

use rand::Rng;

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
    pub fn random_unit() -> Self {
        let mut rng = rand::thread_rng();
        let v = Vector::new(rng.gen(), rng.gen(), rng.gen());
        if v.length_square() > 0. {
            v.normalize()
        } else {
            Vector::random_unit()
        }
    }
    pub fn random_reflection(self) -> Self {
        let v = Vector::random_unit();
        let dp = self.dot(v);
        if dp > 0. {
            v
        } else {
            v - 2. * dp * self
        }
    }
}

pub type Point = Vector;

#[derive(Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vector,
}

impl Ray {
    pub const fn new(origin: Point, direction: Vector) -> Self {
        Ray { origin, direction }
    }
    pub fn at(&self, t: f32) -> Point {
        self.origin + t * self.direction
    }
}

#[derive(Debug)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub const RENDER_RANGE: Self = Interval::new(0.001, f32::INFINITY);
    pub const fn new(min: f32, max: f32) -> Self {
        Interval { min, max }
    }
    pub fn size(&self) -> f32 {
        self.max - self.min
    }
    pub fn contains(&self, x: f32) -> bool {
        self.min <= x && self.max >= x
    }
    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && self.max > x
    }
    pub fn clamp(&self, x: f32) -> f32 {
        x.clamp(self.min, self.max)
    }
}

pub struct Hit {
    pub t: f32,
    pub normal: Vector,
    pub is_front: bool,
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: &Interval) -> Option<Hit>;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f32,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: &Interval) -> Option<Hit> {
        let l = self.center - ray.origin;
        let tca = l.dot(ray.direction);
        let d2 = l.length_square() - tca * tca;
        let r2 = self.radius * self.radius;
        if d2 > r2 {
            None
        } else {
            let thc = (r2 - d2).sqrt();
            let t0 = tca - thc;
            let t1 = tca + thc;
            if interval.surrounds(t0) {
                Some(Hit {
                    t: t0,
                    normal: (ray.at(t0) - self.center).normalize(),
                    is_front: true,
                })
            } else if interval.surrounds(t1) {
                Some(Hit {
                    t: t1,
                    normal: (self.center - ray.at(t1)).normalize(),
                    is_front: false,
                })
            } else {
                None
            }
        }
    }
}
