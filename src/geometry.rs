use std::{
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
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

impl Mul for Vector {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
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
        loop {
            let mut rng = rand::thread_rng();
            let v = Vector::new(rng.gen(), rng.gen(), rng.gen());
            if v.length_square() > 0. && v.length_square() < 1. {
                return v.normalize();
            }
        }
    }
    pub fn near_zero(self) -> bool {
        self.x.abs() < 1e-8 && self.y.abs() < 1e-8 && self.z.abs() < 1e-8
    }
    pub fn reflect(self, normal: Self) -> Self {
        self - 2. * self.dot(normal) * normal
    }
    pub fn refract(self, normal: Self, etai_over_etat: f32) -> Self {
        let cos_theta = f32::min((-1. * self).dot(normal), 1.);
        let r_out_perp = etai_over_etat * (self + cos_theta * normal);
        let r_out_parallel = -(1. - r_out_perp.length_square()).abs().sqrt() * normal;
        r_out_perp + r_out_parallel
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
    pub const RENDER_RANGE: Self = Interval::new(1e-4, f32::INFINITY);
    pub const fn new(min: f32, max: f32) -> Self {
        Interval { min, max }
    }
    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && self.max > x
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

fn color(r: f32, g: f32, b: f32) -> u32 {
    let red = (r.clamp(0., 1.) * 255.) as u32;
    let green = (g.clamp(0., 1.) * 255.) as u32;
    let blue = (b.clamp(0., 1.) * 255.) as u32;
    red << 16 | green << 8 | blue
}

pub type Color = Vector;

pub fn gamma(c: Color) -> u32 {
    color(c.x.sqrt(), c.y.sqrt(), c.z.sqrt())
}

pub enum OnHit {
    None,
    Scatter { attenuation: Color, scattered: Ray },
    Emitted { color: Color },
}

pub trait Material {
    fn on_hit(&self, ray: &Ray, rec: &Hit) -> OnHit;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn on_hit(&self, ray: &Ray, rec: &Hit) -> OnHit {
        let scatter_direction = rec.normal + Vector::random_unit();
        let scatter_direction = if scatter_direction.near_zero() {
            rec.normal
        } else {
            scatter_direction
        };
        OnHit::Scatter {
            attenuation: self.albedo,
            scattered: Ray {
                origin: ray.at(rec.t),
                direction: scatter_direction.normalize(),
            },
        }
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: fuzz.clamp(0., 1.),
        }
    }
}

impl Material for Metal {
    fn on_hit(&self, ray: &Ray, rec: &Hit) -> OnHit {
        let reflected = ray.direction.reflect(rec.normal) + self.fuzz * Vector::random_unit();
        if reflected.dot(rec.normal) > 0. {
            OnHit::Scatter {
                attenuation: self.albedo,
                scattered: Ray {
                    origin: ray.at(rec.t),
                    direction: reflected,
                },
            }
        } else {
            OnHit::None
        }
    }
}

pub struct Dielectric {
    refraction_index: f32,
}

impl Dielectric {
    pub fn new(refraction_index: f32) -> Self {
        Self { refraction_index }
    }
    fn reflectance(cosine: f32, refraction_index: f32) -> f32 {
        let r0 = (1. - refraction_index) / (1. + refraction_index);
        let r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn on_hit(&self, ray: &Ray, rec: &Hit) -> OnHit {
        let ri = if rec.is_front {
            1. / self.refraction_index
        } else {
            self.refraction_index
        };
        let cos_theta = f32::min((-1. * ray.direction).dot(rec.normal), 1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.;
        OnHit::Scatter {
            attenuation: Color::new(1., 1., 1.),
            scattered: Ray {
                origin: ray.at(rec.t),
                direction: if cannot_refract
                    || Dielectric::reflectance(cos_theta, ri) > thread_rng().gen()
                {
                    ray.direction.reflect(rec.normal)
                } else {
                    ray.direction.refract(rec.normal, ri)
                },
            },
        }
    }
}

pub struct Light {
    color: Color,
}

impl Light {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Material for Light {
    fn on_hit(&self, _ray: &Ray, _rec: &Hit) -> OnHit {
        OnHit::Emitted { color: self.color }
    }
}

pub struct Object {
    pub shape: Box<dyn Hittable + Sync>,
    pub material: Arc<dyn Material + Sync + Send>,
}
