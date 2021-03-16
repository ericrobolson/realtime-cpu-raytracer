use super::*;
pub struct World {
    items: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn add(&mut self, item: Box<dyn Hittable>) {
        self.items.push(item);
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut rec = None;
        let mut closest_so_far = t_max;

        for hittable in &self.items {
            match hittable.hit(ray, t_min, t_max) {
                Some(hr) => {
                    if closest_so_far > hr.t {
                        closest_so_far = hr.t;
                        rec = Some(hr);
                    }
                }
                None => {}
            }
        }

        rec
    }
}
