#[path = "collider.rs"] mod collider;
pub use collider::*;


pub enum PDFType {
    CosinePDF(ONB),
    CollidablePDF(Geometry, Point3),
    MixturePDF(Box<PDF>, Box<PDF>)
}


pub struct PDF {
    pub pdf_type: PDFType,
}

impl PDF {
    pub fn cosine_pdf(w: Vec3) -> Self {
        Self {
            pdf_type: PDFType::CosinePDF(ONB::build_from_w(w))
        }
    }
    pub fn collidable_pdf(geometry: Geometry, origin: Point3) -> Self {
        Self {
            pdf_type: PDFType::CollidablePDF(geometry, origin)
        }
    }
    pub fn mixture_pdf(p0: PDF, p1: PDF) -> Self {
        Self {
            pdf_type: PDFType::MixturePDF(Box::new(p0), Box::new(p1))
        }
    }

    pub fn value_cosine(&self, direction: Vec3, uvw: &ONB) -> f32 {
        let cosin = direction.normalize().dot(uvw.w);
        if cosin <= 0.0 { 0.0 } else { cosin / PI }
    }
    pub fn generate_cosine(&self, uvw: &ONB) -> Vec3 {
        uvw.local(random_cosin_direction())
    }

    pub fn value_collidable(&self, geometry: &Geometry, origin: Point3, direction: Vec3) -> f32 {
        geometry.pdf_value(origin, direction)
    }
    pub fn generate_collidable(&self, geometry: &Geometry, origin: Point3) -> Vec3 {
        geometry.random(origin)
    }

    pub fn value_mixture(&self, p0: &Box<PDF>, p1: &Box<PDF>, direction: Vec3) -> f32 {
        return 0.5 * p0.value(direction) + 0.5 * p1.value(direction);
    }
    pub fn generate_mixture(&self, p0: &Box<PDF>, p1: &Box<PDF>) -> Vec3 {
        return if random() < 0.5 { p0.generate() } else { p1.generate() };
    }

    pub fn value(&self, direction: Vec3) -> f32 {
        match &self.pdf_type {
            PDFType::CosinePDF(uvw) => self.value_cosine(direction, uvw),
            PDFType::CollidablePDF(geometry, origin) => self.value_collidable(geometry, *origin, direction),
            PDFType::MixturePDF(p0, p1) => self.value_mixture(p0, p1, direction),
            _ => 0.0,
        }
    }
    pub fn generate(&self) -> Vec3 {
        match &self.pdf_type {
            PDFType::CosinePDF(uvw) => self.generate_cosine(uvw),
            PDFType::CollidablePDF(geometry, origin) => self.generate_collidable(geometry, *origin),
            PDFType::MixturePDF(p0, p1) => self.generate_mixture(p0, p1),
            _ => Vec3::origin(),
        }
    }
}