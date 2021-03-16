use super::{ray::Ray, rng};

type Num = f32;

pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: Num,
    pub y: Num,
    pub z: Num,
}

fn sqr(n: Num) -> Num {
    n * n
}

impl Vec3 {
    pub fn new(x: Num, y: Num, z: Num) -> Self {
        Self { x, y, z }
    }

    pub fn unit_y() -> Self {
        (0., 1., 0.).into()
    }

    pub fn len(&self) -> Num {
        self.len_sqrd().sqrt()
    }

    pub fn len_sqrd(&self) -> Num {
        return sqr(self.x) + sqr(self.y) + sqr(self.z);
    }

    pub fn dot(&self, rhs: Vec3) -> Num {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
    pub fn cross(&self, rhs: Vec3) -> Vec3 {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// A normalized vector of length 1
    pub fn unit_vector(&self) -> Vec3 {
        *self / self.len()
    }

    pub fn random() -> Self {
        Self {
            x: rng::random(),
            y: rng::random(),
            z: rng::random(),
        }
    }

    pub fn random_range(min: f32, max: f32) -> Self {
        Self {
            x: rng::random_range(min, max),
            y: rng::random_range(min, max),
            z: rng::random_range(min, max),
        }
    }

    pub fn random_in_unit_sphere() -> Self {
        const MAX_ITERATIONS: u8 = 10;
        for _ in 0..MAX_ITERATIONS {
            let p = Self::random_range(-1., 1.);
            if p.len_sqrd() <= 1. {
                return p;
            }
        }

        Self::random().unit_vector()
    }

    pub fn random_unit_vector() -> Self {
        return Self::random_in_unit_sphere().unit_vector();
    }

    pub fn random_in_hemisphere(normal: Self) -> Self {
        let in_unit_sphere = Self::random_in_unit_sphere();
        if in_unit_sphere.dot(normal) > 0. {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    pub fn near_zero(&self) -> bool {
        let near_zero: f32 = 1e-8;
        self.x.abs() < near_zero && self.y.abs() < near_zero && self.z.abs() < near_zero
    }

    pub fn reflect(&self, n: Self) -> Self {
        *self - 2. * self.dot(n) * n
    }

    pub fn refract(&self, n: Self, etai_over_etat: f32) -> Self {
        let uv = *self;
        let cos_theta = (-uv).dot(n).min(1.);
        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_parallel = -((1. - r_out_perp.len_sqrd()).abs()) * n;
        r_out_perp + r_out_parallel
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl Into<Vec3> for (f32, f32, f32) {
    fn into(self) -> Vec3 {
        Vec3::new(self.0, self.1, self.2)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::MulAssign<Num> for Vec3 {
    fn mul_assign(&mut self, rhs: Num) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl std::ops::DivAssign<Num> for Vec3 {
    fn div_assign(&mut self, rhs: Num) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}
impl std::ops::Div<Num> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Num) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl std::ops::Mul<Num> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Num) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
impl std::ops::Mul<Vec3> for Num {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}
impl std::ops::Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}
