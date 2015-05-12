//! Defines the scene struct which contains the various objects defining the scene.
//! This includes the geometry, instances of the geometry, the camera and so on.

use std::sync::Arc;
use std::path::Path;

use linalg::{Transform, Point, Vector, Ray};
use film::{Camera, Colorf};
use geometry::{Sphere, Plane, Instance, BoundableGeom, Intersection, Mesh};
use material::{Matte, SpecularMetal, Glass, Material, Merl};
use integrator::{self, Integrator};
use light::{self, Light};
use geometry::BVH;

/// The scene containing the objects and camera configuration we'd like to render,
/// shared immutably among the ray tracing threads
pub struct Scene {
    pub camera: Camera,
    pub bvh: BVH<Instance>,
    pub integrator: Arc<Box<Integrator + Send + Sync>>,
    /// TODO: Only one light for now
    pub light: Arc<Box<Light + Send + Sync>>,
}

impl Scene {
    /// Create our (currently) hard-coded scene, passing in the render target
    /// dimensions so we can set the projection matrix for the camera
    pub fn new(w: usize, h: usize) -> Scene {
        let sphere = Arc::new(Box::new(Sphere::new(1.0)) as Box<BoundableGeom + Send + Sync>);
        let plane = Arc::new(Box::new(Plane) as Box<BoundableGeom + Send + Sync>);

        let white_wall = Arc::new(Box::new(Matte::new(&Colorf::new(0.740063, 0.742313, 0.733934), 1.0))
                                  as Box<Material + Send + Sync>);
        let red_wall = Arc::new(Box::new(Matte::new(&Colorf::new(0.366046, 0.0371827, 0.0416385), 1.0))
                                as Box<Material + Send + Sync>);
        let blue_wall = Arc::new(Box::new(Matte::new(&Colorf::new(0.162928, 0.408903, 0.0833759), 1.0))
                                 as Box<Material + Send + Sync>);

        let mut models = Mesh::load_obj(Path::new("./rust-logo.obj"));
        let rust_logo = Arc::new(Box::new(models.remove("rust_logo").unwrap()) as Box<BoundableGeom + Send + Sync>);

        let mut models = Mesh::load_obj(Path::new("./buddha.obj"));
        let buddha = Arc::new(Box::new(models.remove("buddha").unwrap()) as Box<BoundableGeom + Send + Sync>);

        let mut models = Mesh::load_obj(Path::new("./dragon.obj"));
        let dragon = Arc::new(Box::new(models.remove("dragon").unwrap()) as Box<BoundableGeom + Send + Sync>);

        let oxidized_steel = Arc::new(Box::new(Merl::load_file(Path::new("black-oxidized-steel.binary")))
                                      as Box<Material + Send + Sync>);
        let gold_paint = Arc::new(Box::new(Merl::load_file(Path::new("gold-metallic-paint.binary")))
                                  as Box<Material + Send + Sync>);
        let blue_acrylic = Arc::new(Box::new(Merl::load_file(Path::new("blue-acrylic.binary")))
                                    as Box<Material + Send + Sync>);

        let instances = vec![
            Instance::new(rust_logo, oxidized_steel, Transform::translate(&Vector::new(0.0, 6.0, 12.0))
                          * Transform::rotate_z(180.0) * Transform::rotate_y(90.0)
                          * Transform::scale(&Vector::broadcast(12.0)), "rust_logo"),
            Instance::new(buddha, gold_paint, Transform::translate(&Vector::new(-9.0, 0.5, 7.55))
                          * Transform::rotate_z(-140.0) * Transform::rotate_x(90.0)
                          * Transform::scale(&Vector::broadcast(17.0)), "buddha"),
            Instance::new(dragon, blue_acrylic, Transform::translate(&Vector::new(8.25, 3.5, 3.7))
                          * Transform::rotate_z(120.0) * Transform::rotate_x(90.0)
                          * Transform::scale(&Vector::broadcast(13.0)), "dragon"),
            // The back wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 20.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(90.0), "back_wall"),
            // The left wall
            Instance::new(plane.clone(), red_wall.clone(), Transform::translate(&Vector::new(-15.0, 0.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(90.0), "left_wall"),
            // The right wall
            Instance::new(plane.clone(), blue_wall.clone(), Transform::translate(&Vector::new(15.0, 0.0, 12.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_y(-90.0), "right_wall"),
            // The top wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 0.0, 24.0))
                          * Transform::scale(&Vector::broadcast(32.0)) * Transform::rotate_x(180.0), "top_wall"),
            // The bottom wall
            Instance::new(plane.clone(), white_wall.clone(), Transform::translate(&Vector::new(0.0, 0.0, 0.0))
                          * Transform::scale(&Vector::broadcast(32.0)), "bottom_wall"),
            /*
            // The reflective sphere
            Instance::new(sphere.clone(),
            Arc::new(Box::new(SpecularMetal::new(&Colorf::new(0.155, 0.116, 0.138), &Colorf::new(4.828, 3.122, 2.146)))
                     as Box<Material + Send + Sync>), Transform::translate(&Vector::new(-6.0, 8.0, 5.0))
                    * Transform::scale(&Vector::broadcast(5.0)), "metal_sphere"),
            // The glass sphere
            Instance::new(sphere.clone(),
            Arc::new(Box::new(Glass::new(&Colorf::broadcast(1.0), &Colorf::broadcast(1.0), 1.52))
                     as Box<Material + Send + Sync>), Transform::translate(&Vector::new(6.0, -2.0, 5.0))
                     * Transform::scale(&Vector::broadcast(5.0)), "glass_sphere")
            */
        ];
        let light_color = Colorf::broadcast(200.0) * Colorf::new(0.780131, 0.780409, 0.775833);
        Scene {
            camera: Camera::new(Transform::look_at(&Point::new(0.0, -50.0, 12.0),
                &Point::new(0.0, 0.0, 12.0), &Vector::new(0.0, 0.0, 1.0)), 30.0, (w, h)),
            bvh: BVH::new(4, instances),
            integrator: Arc::new(Box::new(integrator::Path::new(4, 8)) as Box<Integrator + Send + Sync>),
            light: Arc::new(Box::new(light::Point::new(&Point::new(0.0, -4.0, 21.0), &light_color))
                            as Box<Light + Send + Sync>),
        }
    }
    /// Test the ray for intersections against the objects in the scene.
    /// Returns Some(Intersection) if an intersection was found and None if not.
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        self.bvh.intersect(ray, |r, i| i.intersect(r))
    }
}

