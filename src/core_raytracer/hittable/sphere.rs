use super::*;
use crate::core_raytracer::vec3::Point3;

pub struct Sphere {
    center: Point3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin() - self.center;
        let a = ray.direction().len_sqrd();
        let half_b = oc.dot(ray.direction());
        let c = oc.len_sqrd() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        // Find nearest root in the range
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_max || t_max < root {
                return None;
            }
        }
        let t = root;
        let point = ray.at(t);
        let outward_normal: Vec3 = (point - self.center) / self.radius;

        Some(HitRecord::new(point, ray, outward_normal, t, self.material))
    }
}

fn hit_sphere(center: &Point3, radius: f32, ray: &Ray) -> f32 {
    let oc = ray.origin() - *center;
    let a = ray.direction().len_sqrd();
    let half_b = oc.dot(ray.direction());
    let c = oc.len_sqrd() - radius * radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0. {
        -1.
    } else {
        (-half_b - discriminant.sqrt()) / a
    }
}
