#[path = "ray.rs"] mod ray;
pub use ray::*;


pub enum PDFType {
    CosinePDF(ONB),
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


    pub fn value_cosine(&self, direction: Vec3, uvw: &ONB) -> f32 {
        let cosin = direction.normalize().dot(uvw.w);
        if cosin <= 0.0 { 0.0 } else { cosin / PI }
    }
    pub fn generate_cosine(&self, uvw: &ONB) -> Vec3 {
        uvw.local(random_cosin_direction())
    }

    pub fn value(&self, direction: Vec3) -> f32 {
        match &self.pdf_type {
            PDFType::CosinePDF(uvw) => self.value_cosine(direction, uvw),
            _ => 0.0,
        }
    }
    pub fn generate(&self) -> Vec3 {
        match &self.pdf_type {
            PDFType::CosinePDF(uvw) => self.generate_cosine(uvw),
            _ => Vec3::origin(),
        }
    }
}