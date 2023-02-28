#[path = "render.rs"] mod render;
use render::*;
use macroquad::prelude::{Image, Texture2D, screen_width, screen_height, Vec2, Color, draw_texture_ex, DrawTextureParams, next_frame};


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

    let mat_ground = Material::lambertian(Texture::solid_color(Rgb::new(0.8, 0.8, 0.0)));
    let mat_center = Material::lambertian(Texture::solid_color(Rgb::new(0.1, 0.2, 0.5)));
    let mat_left = Material::dielectric(1.5);
    let mat_left_in = Material::dielectric(1.5);
    let mat_right = Material::glossy(Rgb::new(0.8, 0.6, 0.2), 0.4);

    scene.add(Geometry::sphere(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground));
    scene.add(Geometry::sphere(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center));
    scene.add(Geometry::sphere(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left));
    scene.add(Geometry::sphere(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left_in));
    scene.add(Geometry::sphere(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right));

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
    let mut spheres: Vec<Geometry> = Vec::new();

    let mat_ground = Material::lambertian(Texture::checkered(Rgb::new(0.2, 0.3, 0.1), Rgb::new(0.9, 0.9, 0.9)));
    scene.add(Geometry::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, mat_ground));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random();
            let center = Point3::new(a as f32 + 0.9 * random(), 0.2, b as f32 + 0.9 * random());
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Rgb::random() * Rgb::random();
                    let mat = Material::lambertian(Texture::solid_color(albedo));
                    spheres.push(Geometry::sphere(center, 0.2, mat));
                } else if choose_mat < 0.95 {
                    let albedo = Rgb::randrange(0.5, 1.0);
                    let fuzz = randrange(0.0, 0.5);
                    let mat = Material::glossy(albedo, fuzz);
                    spheres.push(Geometry::sphere(center, 0.2, mat));
                } else {
                    let mat = Material::dielectric(1.5);
                    spheres.push(Geometry::sphere(center, 0.2, mat));
                }
            }
        }
    }

    let mat1 = Material::dielectric(1.5);
    spheres.push(Geometry::sphere(Point3::new(0.0, 1.0, 0.0), 1.0, mat1));
    
    let mat2 = Material::lambertian(Texture::solid_color(Rgb::new(0.4, 0.2, 0.1)));
    spheres.push(Geometry::sphere(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2));
    
    let mat3 = Material::glossy(Rgb::new(0.7, 0.6, 0.5), 0.1);
    spheres.push(Geometry::sphere(Point3::new(4.0, 1.0, 0.0), 1.0, mat3));

    scene.add(Geometry::bvh_node(&spheres, 0.0, 1.0, 0, spheres.len()));
    
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
    let mut spheres: Vec<Geometry> = Vec::new();

    let mat_ground = Material::lambertian(Texture::solid_color(Rgb::new(0.5, 0.5, 0.5)));
    scene.add(Geometry::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, mat_ground));

    for a in -21..21 {
        for b in -21..21 {
            let choose_mat = random();
            let center = Point3::new(a as f32 / 2.0 + 0.9 * random(), 0.2, b as f32 / 2.0 + 0.9 * random());
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Rgb::random() * Rgb::random();
                    let mat = Material::lambertian(Texture::solid_color(albedo));
                    let center2 = center + Vec3::new(0.0, randrange(0.0, 0.5), 0.0);
                    spheres.push(Geometry::moving_sphere(center, center2, 0.0, 1.0, 0.1, mat));
                } else if choose_mat < 0.95 {
                    let albedo = Rgb::randrange(0.5, 1.0);
                    let fuzz = randrange(0.0, 0.5);
                    let mat = Material::glossy(albedo, fuzz);
                    spheres.push(Geometry::sphere(center, 0.1, mat));
                } else {
                    let mat = Material::dielectric(1.5);
                    spheres.push(Geometry::sphere(center, 0.1, mat));
                }
            }
        }
    }
    scene.add(Geometry::bvh_node(&spheres, 0.0, 1.0, 0, spheres.len()));

    let mat1 = Material::dielectric(1.5);
    scene.add(Geometry::sphere(Point3::new(0.0, 1.0, 0.0), 1.0, mat1));
    
    let mat2 = Material::lambertian(Texture::solid_color(Rgb::new(0.4, 0.2, 0.1)));
    scene.add(Geometry::sphere(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2));
    
    let mat3 = Material::glossy(Rgb::new(0.7, 0.6, 0.5), 0.1);
    scene.add(Geometry::sphere(Point3::new(4.0, 1.0, 0.0), 1.0, mat3));
    
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

    let checker = Material::lambertian(Texture::checkered(Rgb::new(0.2, 0.3, 0.1), Rgb::new(0.9, 0.9, 0.9)));
    scene.add(Geometry::sphere(Point3::new(0.0, -8.0, 0.0), 8.0, checker.clone()));
    scene.add(Geometry::sphere(Point3::new(0.0, 8.0, 0.0), 8.0, checker.clone()));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn two_perlin_spheres() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 600;
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

    let pertext = Material::lambertian(Texture::noise(4.0, 7));
    scene.add(Geometry::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, pertext.clone()));
    scene.add(Geometry::sphere(Point3::new(0.0, 2.0, 0.0), 2.0, pertext.clone()));

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

    let earth_surface = Material::lambertian(scene.load_image("assets/earthmap.jpeg"));
    scene.add(Geometry::sphere(Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface));

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

    let texture = Material::lambertian(Texture::solid_color(Rgb::new(0.2, 0.8, 1.0)));
    let checker = Material::lambertian(Texture::solid_color(Rgb::new(0.9, 0.9, 0.9)));
    scene.add(Geometry::sphere(Point3::new(0.0, -1000.0, 0.0), 1000.0, checker));
    scene.add(Geometry::sphere(Point3::new(0.0, 2.0, 0.0), 2.0, texture));

    let difflight = Material::emissive(Rgb::new(5.0, 2.0, 2.0));
    scene.add(Geometry::xyrect(3.0, 5.0, 1.0, 3.0, -2.0, difflight.clone()));
    scene.add(Geometry::sphere(Point3::new(0.0, 7.0, 0.0), 1.5, difflight));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn cornell_box() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 600;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 200;
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

    let red = Material::lambertian(Texture::solid_color(Rgb::new(0.65, 0.05, 0.05)));
    let white = Material::lambertian(Texture::solid_color(Rgb::new(0.73, 0.73, 0.73)));
    let green = Material::lambertian(Texture::solid_color(Rgb::new(0.12, 0.45, 0.15)));
    let light = Material::emissive(Rgb::new(30.0, 30.0, 30.0));

    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 555.0, green));
    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 0.0, red));
    scene.add(Geometry::xzrect(213.0, 343.0, 227.0, 332.0, 554.0, light));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));
    scene.add(Geometry::xyrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));

    scene.add(Geometry::instance_translation(Geometry::instance_rotation(Geometry::cuboid(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone()), Axis::Y, 15.0), Vec3::new(265.0, 0.0, 295.0)));
    scene.add(Geometry::instance_translation(Geometry::instance_rotation(Geometry::cuboid(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone()), Axis::Y, -18.0), Vec3::new(130.0, 0.0, 65.0)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn cornell_smoke() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 600;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 200;
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

    let red = Material::lambertian(Texture::solid_color(Rgb::new(0.65, 0.05, 0.05)));
    let white = Material::lambertian(Texture::solid_color(Rgb::new(0.73, 0.73, 0.73)));
    let green = Material::lambertian(Texture::solid_color(Rgb::new(0.12, 0.45, 0.15)));
    let light = Material::emissive(Rgb::new(7.0, 7.0, 7.0));

    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 555.0, green));
    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 0.0, red));
    scene.add(Geometry::xzrect(113.0, 443.0, 127.0, 432.0, 554.0, light));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));
    scene.add(Geometry::xyrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));

    scene.add(Geometry::constant_medium(Geometry::instance_translation(Geometry::instance_rotation(Geometry::cuboid(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone()), Axis::Y, 15.0), Vec3::new(265.0, 0.0, 295.0)), 0.01, Rgb::new(0.0, 0.0, 0.0)));
    scene.add(Geometry::constant_medium(Geometry::instance_translation(Geometry::instance_rotation(Geometry::cuboid(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone()), Axis::Y, -18.0), Vec3::new(130.0, 0.0, 65.0)), 0.01, Rgb::new(1.0, 1.0, 1.0)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn cornell_pedestal() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 600;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 200;
    let max_depth = 100;
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

    let red = Material::lambertian(Texture::solid_color(Rgb::new(0.65, 0.05, 0.05)));
    let white = Material::lambertian(Texture::solid_color(Rgb::new(0.73, 0.73, 0.73)));
    let green = Material::lambertian(Texture::solid_color(Rgb::new(0.12, 0.45, 0.15)));
    let light = Material::emissive(Rgb::new(5.0, 5.0, 5.0));
    let light_powerful = Material::emissive(Rgb::new(60.0, 60.0, 60.0));

    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 555.0, green));
    scene.add(Geometry::yzrect(0.0, 555.0, 0.0, 555.0, 0.0, red));
    scene.add(Geometry::xzrect(113.0, 443.0, 127.0, 432.0, 554.0, light.clone()));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 0.0, white.clone()));
    scene.add(Geometry::xzrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));
    scene.add(Geometry::xyrect(0.0, 555.0, 0.0, 555.0, 555.0, white.clone()));

    scene.add(Geometry::xyrect(113.0, 443.0, 127.0, 432.0, -850.0, light_powerful.clone()));

    scene.add(Geometry::instance_translation(Geometry::instance_rotation(Geometry::cuboid(Point3::new(0., 0., 0.), Point3::new(125., 125., 125.), white.clone()), Axis::Y, 45.0), Vec3::new(188., 0., 178.)));
    
    // scene.add(Geometry::triangle(Point3::new(200., 100., 400.), Point3::new(100., 100., 200.), Point3::new(100., 200., 400.), white.clone()));
    scene.add(Geometry::instance_translation(Geometry::instance_rotation(Geometry::instance_rotation(Geometry::load_obj("assets/objs/suzanne.obj", 80.0, white.clone()), Axis::Y, 145.), Axis::Z, -30.), Vec3::new(270., 200., 178.)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}

fn final_scene() -> (Camera, SceneColliders, Rgb, f32, u32, u32, usize, usize) {
    // Image
    let aspect_ratio = 1.0;
    let image_width = 800;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let samples_per_pixel = 300;
    let max_depth = 50;
    let background = Rgb::new(0.0, 0.0, 0.0);

    // Camera
    let look_from = Point3::new(478.0, 278.0, -600.0);
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

    let ground = Material::lambertian(Texture::solid_color(Rgb::new(0.48, 0.83, 0.53)));
    let boxes_per_side = 20;
    let mut boxes: Vec<Geometry> = Vec::new();

    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f32 * w;
            let z0 = -1000.0 + j as f32 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = randrange(1.0, 101.0);
            let z1 = z0 + w;

            boxes.push(Geometry::cuboid(Point3::new(x0, y0, z0), Point3::new(x1, y1, z1), ground.clone()))
        }
    }
    scene.add(Geometry::bvh_node(&boxes, 0.0, 1.0, 0, boxes.len()));
    
    let light = Material::emissive(Rgb::new(10.0, 10.0, 10.0));
    scene.add(Geometry::xzrect(123.0, 423.0, 147.0, 412.0, 554.0, light.clone()));

    scene.add(Geometry::sphere(Point3::new(260.0, 150.0, 45.0), 50.0, Material::dielectric(1.5)));
    scene.add(Geometry::sphere(Point3::new(0.0, 150.0, 145.0), 50.0, Material::glossy(Rgb::new(0.8, 0.8, 0.9), 1.0)));

    let boundary = Geometry::sphere(Point3::new(360.0, 150.0, 145.0), 70.0, Material::dielectric(1.5));
    scene.add(boundary.clone());
    scene.add(Geometry::constant_medium(boundary, 0.2, Rgb::new(0.2, 0.4, 0.9)));
    let boundary = Geometry::sphere(Point3::origin(), 5000.0, Material::dielectric(1.5));
    scene.add(Geometry::constant_medium(boundary, 0.0001, Rgb::new(1.0, 1.0, 1.0)));

    let emat = Material::lambertian(scene.load_image("assets/earthmap.jpeg"));
    scene.add(Geometry::sphere(Point3::new(400.0, 200.0, 400.0), 100.0, emat));
    // let emat = Material::glossy(Rgb::new(0.8, 0.8, 0.8), 0.0);
    // scene.add(Geometry::sphere(Point3::new(400.0, 200.0, 400.0), 100.0, emat));
    let pertext = Texture::noise(0.1, 7);
    scene.add(Geometry::sphere(Point3::new(220.0, 280.0, 300.0), 80.0, Material::lambertian(pertext)));

    let mut spheres: Vec<Geometry> = Vec::new();
    let white = Material::lambertian(Texture::solid_color(Rgb::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        spheres.push(Geometry::sphere(Point3::randrange(0.0, 165.0), 10.0, white.clone()));
    }
    scene.add(Geometry::instance_translation(Geometry::instance_rotation(Geometry::bvh_node(&spheres, 0.0, 1.0, 0, spheres.len()), Axis::Y, 15.0), Vec3::new(-100.0, 270.0, 395.0)));

    return (cam, scene, background, aspect_ratio, image_width, image_height, samples_per_pixel, max_depth);
}


// fn main() {
//     let (cam, scene, background, _aspect_ratio, img_width, img_height, samples_per_pixel, max_depth) = cornell_box();
//     let imgbuf = render_multi(
//         scene,
//         cam,
//         background,
//         max_depth,
//         samples_per_pixel,
//         img_width, 
//         img_height,
//     );

//     imgbuf.save(&format!("output/{}", FILENAME)).unwrap();
// }


#[macroquad::main("Rust Raytracer")]
async fn main() {
    let (cam, scene, background, aspect_ratio, img_width, img_height, samples_per_pixel, max_depth) = cornell_box();

    let mut img = Image::gen_image_color(img_width as u16, img_height as u16, BLACK);
    let mut img_full = Vec::new();
    for _ in 0..img_width {
        let mut col = Vec::new();
        for _ in 0..img_height {
            col.push([0f32; 3]);
        }
        img_full.push(col);
    }

    let mut iterations = 0;
    let texture = Texture2D::from_image(&img);
    let samples_per_frame = num_cpus::get() * 1;
    loop {
        iterations += 1;
        // clear_background(BLACK);

        let imgbuf = render_frame_multi(
            scene.clone(),
            cam.clone(),
            background,
            max_depth,
            samples_per_frame,
            img_width, 
            img_height,
        );
        
        for (x, y, pixel) in imgbuf.enumerate_pixels() {
            let old_val = img_full[x as usize][y as usize];
            let r = old_val[0] + pixel.0[0];
            let g = old_val[1] + pixel.0[1];
            let b = old_val[2] + pixel.0[2];
            img_full[x as usize][y as usize] = [r, g, b];
            img.set_pixel(x, y, Color::new(r / iterations as f32, g / iterations as f32, b / iterations as f32, 1.0));
        }

        texture.update(&img);
        let width_min = screen_width() < screen_height();
        let pos = if width_min {
            Vec2::new(screen_width(), screen_width() / aspect_ratio)
        } else {
            Vec2::new(screen_height() * aspect_ratio, screen_height())
        };
        draw_texture_ex(
            texture,
            0.0, 0.0,
            Color::new(1.0, 1.0, 1.0, 1.0), 
            DrawTextureParams{dest_size: Some(pos), ..Default::default()}
        );
        println!("{} samples per pixel", iterations * samples_per_frame);
        next_frame().await
    }
}
