use super::{
    hittable::HitRecord,
    ray::Ray,
    rng,
    vec3::{Color, Vec3},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Material {
    Lambertian {
        albedo: Color,
    },
    Metal {
        albedo: Color,
        fuzz: f32,
    },
    Dielectric {
        /// Index of refraction
        ior: f32,
    },
}

impl Material {
    pub fn scatter(&self, ray: &Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Material::Lambertian { albedo } => {
                let scatter_dir = {
                    let scattered_dir = rec.normal + Vec3::random_unit_vector();
                    // Handle degenerate scatter direction
                    if scattered_dir.near_zero() {
                        rec.normal
                    } else {
                        scattered_dir
                    }
                };
                let scattered = Ray::new(rec.point, scatter_dir);
                let attenuation = *albedo;
                return Some((scattered, attenuation));
            }
            Material::Metal { albedo, fuzz } => {
                let fuzz = {
                    if *fuzz < 1. {
                        *fuzz
                    } else {
                        1.
                    }
                };

                let reflected = ray.direction().unit_vector().reflect(rec.normal);
                let scattered =
                    Ray::new(rec.point, reflected + fuzz * Vec3::random_in_unit_sphere());
                let attenuation = *albedo;
                if scattered.direction().dot(rec.normal) > 0. {
                    return Some((scattered, attenuation));
                }
            }
            Material::Dielectric { ior } => {
                let attenuation: Color = (1., 1., 1.).into();
                let refraction_ratio = if rec.front_face { 1. / ior } else { *ior };

                let unit_dir = ray.direction().unit_vector();
                let cos_theta = (-unit_dir).dot(rec.normal).min(1.);
                let sin_theta = (1. - cos_theta * cos_theta).sqrt();
                let cannot_refract = refraction_ratio * sin_theta > 1.;

                let direction =
                    if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng::random() {
                        unit_dir.reflect(rec.normal)
                    } else {
                        unit_dir.refract(rec.normal, refraction_ratio)
                    };

                let scattered = Ray::new(rec.point, direction);
                return Some((scattered, attenuation));
            }
        }

        None
    }
}

fn reflectance(cos: f32, ref_idx: f32) -> f32 {
    let r0 = (1. - ref_idx) / (1. + ref_idx);
    let r0 = r0 * r0;
    r0 + (1. - r0) * (1. - cos).powi(5)
}
