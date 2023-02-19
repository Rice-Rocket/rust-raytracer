use std::sync::Arc;
use image::{self, ImageBuffer};
#[path = "render.rs"] mod render;
use render::*;


const FILENAME: &str = "out.png";


fn default_scene() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 500;
    let max_depth = 200;
    let background = Rgb::new(0.7, 0.8, 1.0);

    // Camera
    let look_from = Point3::new(3.0, 3.0, 2.0);
    let look_at = Point3::new(0.0, 0.0, -1.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        30.0, 
        0.1,
        (look_from - look_at).length(),
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let mat_ground = Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.8, 0.8, 0.0))));
    let mat_center = Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.1, 0.2, 0.5))));
    let mat_left = Dielectric::new(1.5);
    let mat_left_in = Dielectric::new(1.5);
    let mat_right = Glossy::new(Rgb::new(0.8, 0.6, 0.2), 0.4);

    scene.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, Arc::new(mat_ground))));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, Arc::new(mat_center))));
    scene.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, Arc::new(mat_left))));
    scene.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, Arc::new(mat_left_in))));
    scene.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, Arc::new(mat_right))));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn random_spheres() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 600;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 250;
    let max_depth = 50;
    let background = Rgb::new(0.7, 0.8, 1.0);

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0, 
        0.1,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();
    let mut spheres: Vec<Arc<dyn Geometry + Send + Sync>> = Vec::new();

    let mat_ground = Lambertian::new(Arc::new(Checkered::new(Rgb::new(0.2, 0.3, 0.1), Rgb::new(0.9, 0.9, 0.9))));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, Arc::new(mat_ground))));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random();
            let center = Point3::new(a as f32 + 0.9 * random(), 0.2, b as f32 + 0.9 * random());
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Rgb::random() * Rgb::random();
                    let mat = Lambertian::new(Arc::new(SolidColor::new(albedo)));
                    spheres.push(Arc::new(Sphere::new(center, 0.2, Arc::new(mat))));
                } else if choose_mat < 0.95 {
                    let albedo = Rgb::randrange(0.5, 1.0);
                    let fuzz = randrange(0.0, 0.5);
                    let mat = Glossy::new(albedo, fuzz);
                    spheres.push(Arc::new(Sphere::new(center, 0.2, Arc::new(mat))));
                } else {
                    let mat = Dielectric::new(1.5);
                    spheres.push(Arc::new(Sphere::new(center, 0.2, Arc::new(mat))));
                }
            }
        }
    }

    let mat1 = Dielectric::new(1.5);
    spheres.push(Arc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Arc::new(mat1))));
    
    let mat2 = Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.4, 0.2, 0.1))));
    spheres.push(Arc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, Arc::new(mat2))));
    
    let mat3 = Glossy::new(Rgb::new(0.7, 0.6, 0.5), 0.1);
    spheres.push(Arc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, Arc::new(mat3))));

    scene.add(Arc::new(BVHNode::new(&spheres, 0.0, 1.0, 0, spheres.len())));
    
    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn random_moving_spheres() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 400;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Rgb::new(0.7, 0.8, 1.0);

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0, 
        0.1,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();
    let mut spheres: Vec<Arc<dyn Geometry + Send + Sync>> = Vec::new();

    let mat_ground = Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.5, 0.5, 0.5))));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, Arc::new(mat_ground))));

    for a in -21..21 {
        for b in -21..21 {
            let choose_mat = random();
            let center = Point3::new(a as f32 / 2.0 + 0.9 * random(), 0.2, b as f32 / 2.0 + 0.9 * random());
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Rgb::random() * Rgb::random();
                    let mat = Lambertian::new(Arc::new(SolidColor::new(albedo)));
                    let center2 = center + Vec3::new(0.0, randrange(0.0, 0.5), 0.0);
                    spheres.push(Arc::new(MovingSphere::new(center, center2, 0.0, 1.0, 0.1, Arc::new(mat))));
                } else if choose_mat < 0.95 {
                    let albedo = Rgb::randrange(0.5, 1.0);
                    let fuzz = randrange(0.0, 0.5);
                    let mat = Glossy::new(albedo, fuzz);
                    spheres.push(Arc::new(Sphere::new(center, 0.1, Arc::new(mat))));
                } else {
                    let mat = Dielectric::new(1.5);
                    spheres.push(Arc::new(Sphere::new(center, 0.1, Arc::new(mat))));
                }
            }
        }
    }
    scene.add(Arc::new(BVHNode::new(&spheres, 0.0, 1.0, 0, spheres.len())));

    let mat1 = Dielectric::new(1.5);
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Arc::new(mat1))));
    
    let mat2 = Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.4, 0.2, 0.1))));
    scene.add(Arc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, Arc::new(mat2))));
    
    let mat3 = Glossy::new(Rgb::new(0.7, 0.6, 0.5), 0.1);
    scene.add(Arc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, Arc::new(mat3))));
    
    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn two_spheres() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Rgb::new(0.7, 0.8, 1.0);

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0, 
        0.0,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let checker = Arc::new(Lambertian::new(Arc::new(Checkered::new(Rgb::new(0.2, 0.3, 0.1), Rgb::new(0.9, 0.9, 0.9)))));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, -8.0, 0.0), 8.0, checker.clone())));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 8.0, 0.0), 8.0, checker.clone())));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn earth() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Rgb::new(0.7, 0.8, 1.0);

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0, 
        0.0,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let earth_texture = ImageTexture::load("assets/earthmap.jpeg");
    let earth_surface = Arc::new(Lambertian::new(Arc::new(earth_texture)));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn rect_light() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 800;
    let max_depth = 50;
    let background = Rgb::new(0.0, 0.0, 0.0);

    // Camera
    let look_from = Point3::new(26.0, 3.0, 6.0);
    let look_at = Point3::new(0.0, 2.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0, 
        0.0,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let texture = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.2, 0.8, 1.0)))));
    let checker = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.9, 0.9, 0.9)))));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, checker)));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 2.0, 0.0), 2.0, texture)));

    let difflight = Arc::new(Emissive::new(Rgb::new(5.0, 2.0, 2.0)));
    scene.add(Arc::new(XYRect::new(3.0, 5.0, 1.0, 3.0, -2.0, difflight.clone())));
    scene.add(Arc::new(Sphere::new(Point3::new(0.0, 7.0, 0.0), 1.5, difflight)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn cornell_box() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 500;
    let max_depth = 200;
    let background = Rgb::new(0.0, 0.0, 0.0);

    // Camera
    let look_from = Point3::new(278.0, 278.0, -800.0);
    let look_at = Point3::new(278.0, 278.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        40.0, 
        0.0,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let red = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.65, 0.05, 0.05)))));
    let white = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.73, 0.73, 0.73)))));
    let green = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.12, 0.45, 0.15)))));
    let light = Arc::new(Emissive::new(Rgb::new(30.0, 30.0, 30.0)));

    scene.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green)));
    scene.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red)));
    scene.add(Arc::new(XZRect::new(213.0, 343.0, 227.0, 332.0, 554.0, light)));
    scene.add(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())));
    scene.add(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));
    scene.add(Arc::new(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    scene.add(Arc::new(TranslateInstance::new(Arc::new(YRotationInstance::new(Arc::new(Cuboid::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone())), 15.0)), Vec3::new(265.0, 0.0, 295.0))));
    scene.add(Arc::new(TranslateInstance::new(Arc::new(YRotationInstance::new(Arc::new(Cuboid::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone())), -18.0)), Vec3::new(130.0, 0.0, 65.0))));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn cornell_smoke() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 500;
    let max_depth = 200;
    let background = Rgb::new(0.0, 0.0, 0.0);

    // Camera
    let look_from = Point3::new(278.0, 278.0, -800.0);
    let look_at = Point3::new(278.0, 278.0, 0.0);

    let cam = Camera::new(
        look_from, 
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        40.0, 
        0.0,
        10.0,
        aspect_ratio,
        0.0, 1.0
    );

    // Scene
    let mut scene = SceneColliders::new();

    let red = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.65, 0.05, 0.05)))));
    let white = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.73, 0.73, 0.73)))));
    let green = Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::new(0.12, 0.45, 0.15)))));
    let light = Arc::new(Emissive::new(Rgb::new(15.0, 15.0, 15.0)));

    scene.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green)));
    scene.add(Arc::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red)));
    scene.add(Arc::new(XZRect::new(213.0, 343.0, 227.0, 332.0, 554.0, light)));
    scene.add(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, white.clone())));
    scene.add(Arc::new(XZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));
    scene.add(Arc::new(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white.clone())));

    scene.add(Arc::new(ConstantMedium::new(Arc::new(TranslateInstance::new(Arc::new(YRotationInstance::new(Arc::new(Cuboid::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone())), 15.0)), Vec3::new(265.0, 0.0, 295.0))), 0.01, Rgb::new(0.0, 0.0, 0.0))));
    scene.add(Arc::new(ConstantMedium::new(Arc::new(TranslateInstance::new(Arc::new(YRotationInstance::new(Arc::new(Cuboid::new(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone())), -18.0)), Vec3::new(130.0, 0.0, 65.0))), 0.01, Rgb::new(1.0, 1.0, 1.0))));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}


fn main() {
    let (cam, scene, background, aspect_ratio, img_width, img_height, samples_per_pixel, max_depth) = cornell_smoke();
    let imgbuf = render_multi(
        scene,
        cam,
        background,
        max_depth,
        samples_per_pixel,
        img_width, 
        img_height,
    );

    imgbuf.save(&format!("output/{}", FILENAME)).unwrap();
}
