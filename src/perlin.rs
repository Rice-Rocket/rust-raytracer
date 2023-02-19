#[path = "ray.rs"] mod ray;
pub use ray::*;


pub struct Perlin {
    point_count: usize,
    rand_vec: Vec<Vec3>,
    perm_x: Vec<isize>,
    perm_y: Vec<isize>,
    perm_z: Vec<isize>
}

impl Perlin {
    pub fn new() -> Self {
        let pointcount = 256;
        let mut rand_vec = Vec::new();
        for _ in 0..pointcount {
            rand_vec.push(Vec3::randrange(-1.0, 1.0).normalize());
        }
        let perm_x = Self::generate_perm(pointcount);
        let perm_y = Self::generate_perm(pointcount);
        let perm_z = Self::generate_perm(pointcount);
        Self {
            point_count: pointcount,
            rand_vec: rand_vec,
            perm_x: perm_x,
            perm_y: perm_y,
            perm_z: perm_z
        }
    }
    fn generate_perm(pointcount: usize) -> Vec<isize> {
        let mut perm = Vec::new();
        for i in 0..pointcount {
            perm.push(i as isize);
        }
        return Self::permute(perm, pointcount);
    }
    fn permute(perm: Vec<isize>, n: usize) -> Vec<isize> {
        let mut new_perm = perm.clone();
        for i in (1..=(n - 1)).rev() {
            let target = randuint(0, i);
            new_perm[i] = perm[target];
            new_perm[target] = perm[i];
        }
        return new_perm;
    }
    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let mut accum = 0.0;
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_vec = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                    accum += (i as f32 * uu + (1.0 - i as f32) * (1.0 - uu)) * (
                              j as f32 * vv + (1.0 - j as f32) * (1.0 - vv)) * (
                              k as f32 * ww + (1.0 - k as f32) * (1.0 - ww)) * c[i][j][k].dot(weight_vec);
                }
            }
        }
        return accum;
    }
    pub fn noise(&self, point: Point3) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let i = point.x.floor() as isize;
        let j = point.y.floor() as isize;
        let k = point.z.floor() as isize;
        let mut c = [[[Vec3::origin(); 2]; 2]; 2];

        for di in 0isize..2isize {
            for dj in 0isize..2isize {
                for dk in 0isize..2isize {
                    c[di as usize][dj as usize][dk as usize] = self.rand_vec[
                        (self.perm_x[((i + di) & 255) as usize] ^ 
                        self.perm_y[((j + dj) & 255) as usize] ^ 
                        self.perm_z[((k + dk) & 255) as usize]) as usize
                    ]
                }
            }
        }
        Self::perlin_interp(c, u, v, w)
    }
    pub fn turb(&self, point: Point3, depth: usize) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = point.clone();
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2.0;
        }
        return accum.abs();
    }
}