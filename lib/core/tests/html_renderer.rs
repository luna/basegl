//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

use web_test::web_configure;
web_configure!(run_in_browser);

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen(module = "/tests/bench_test.js")]
extern "C" {
    fn set_gradient_bg(
        dom   : &JsValue,
        red   : &JsValue,
        green : &JsValue,
        blue  : &JsValue);
}

#[cfg(test)]
mod tests {
    use basegl::display::camera::Camera2d;
    use basegl::system::web::dom::html::Css3dObject;
    use basegl::system::web::dom::html::Css3dRenderer;
    use basegl::system::web::StyleSetter;
    use basegl::system::web::get_performance;
    use web_test::*;
    use web_sys::Performance;
    use nalgebra::Vector3;
    use logger::Logger;
    use basegl::system::web::set_stdout;

    #[web_test(no_container)]
    fn invalid_container() {
        let logger   = Logger::new("invalid_container");
        let renderer = Css3dRenderer::new(logger, "nonexistent_id");
        assert!(renderer.is_err(), "nonexistent_id should not exist");
    }

    fn create_scene(renderer:&Css3dRenderer) -> Vec<Css3dObject> {
        renderer.container().dom.set_property_or_panic("background-color", "black");

        let mut objects = Vec::new();
        // Iterate over 3 axes.
        for axis in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            // Creates 10 HTMLObjects per axis.
            for i in 0 .. 10 {
                let mut object = renderer.new_instance("div").unwrap();
                object.set_dimensions(10.0, 10.0);

                // Using axis for masking.
                // For instance, the axis (0, 1, 0) creates:
                // (x, y, z) = (0, 0, 0) .. (0, 9, 0)
                let x = (i * axis.0) as f32;
                let y = (i * axis.1) as f32;
                let z = (i * axis.2) as f32;
                let factor = 120.0 / 9.0;
                let position = Vector3::new(x * factor + 160.0, y * factor + 120.0, z * factor);
                object.set_position(position);

                // Creates a gradient color based on the axis.
                let r = (x * 25.5) as u8;
                let g = (y * 25.5) as u8;
                let b = (z * 25.5) as u8;
                let color = format!("rgba({}, {}, {}, {})", r, g, b, 1.0);
                object.dom.set_property_or_panic("background-color", color);
                objects.push(object);
            }
        }
        objects
    }

    #[web_test]
    fn rhs_coordinates() {
        set_stdout();
        let logger   = Logger::new("rhs_coordinates");
        let renderer = Css3dRenderer::new(&logger, "rhs_coordinates")
                                    .expect("Renderer couldn't be created");
        let mut scene = create_scene(&renderer);

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut camera  = Camera2d::new(logger,view_dim.x,view_dim.y);

        renderer.render(&mut camera);

        // To remove the x-axis
        for _ in 0..scene.len()/3 {
            scene.remove(0);
        }

        renderer.render(&mut camera);

        std::mem::forget(scene);
    }

    #[web_bench]
    fn camera_movement(b: &mut Bencher) {
        let logger = Logger::new("camera_movement");
        let renderer = Css3dRenderer::new(&logger, "camera_movement")
                                    .expect("Renderer couldn't be created");
        let scene = create_scene(&renderer);

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut camera  = Camera2d::new(logger,view_dim.x,view_dim.y);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        b.iter(move || {
            let _keep_alive = &scene;
            let t = (performance.now() / 1000.0) as f32;
            // We move the Camera 29 units away from the center.
            camera.set_position(Vector3::new(t.sin() * 50.0, t.cos() * 50.0, 200.0));

            renderer.render(&mut camera);
        })
    }

    fn make_sphere(mut scene : &mut Vec<Css3dObject>, performance : &Performance) {
        use super::set_gradient_bg;

        let t = (performance.now() / 1000.0) as f32;
        let length = scene.len() as f32;
        for (i, object) in (&mut scene).into_iter().enumerate() {
            let i = i as f32;
            let d = (i / length - 0.5) * 2.0;

            let mut y = d;
            let r = (1.0 - y * y).sqrt();
            let mut x = (y * 100.0 + t).cos() * r;
            let mut z = (y * 100.0 + t).sin() * r;

            x += (y * 1.25 + t * 2.50).cos() * 0.5;
            y += (z * 1.25 + t * 2.00).cos() * 0.5;
            z += (x * 1.25 + t * 3.25).cos() * 0.5;
            let x = x * 5.0 + 160.0;
            let y = y * 5.0 + 120.0;
            let z = z * 5.0;
            object.set_position(Vector3::new(x, y, z));

            let faster_t = t * 100.0;
            let r = (i +   0.0 + faster_t) as u8 % 255;
            let g = (i +  85.0 + faster_t) as u8 % 255;
            let b = (i + 170.0 + faster_t) as u8 % 255;
            set_gradient_bg(&object.dom, &r.into(), &g.into(), &b.into());
        }
    }

    #[web_bench]
    fn object_x400_update(b: &mut Bencher) {
        let logger = Logger::new("object_x400_update");
        let renderer = Css3dRenderer::new(&logger, "object_x400_update")
                                    .expect("Renderer couldn't be created");
        let mut scene = Vec::new();
        renderer.container().dom.set_property_or_panic("background-color", "black");

        for _ in 0..400 {
            let mut object = renderer.new_instance("div").expect("Failed to create object");
            object.set_dimensions(1.0, 1.0);
            object.set_scale(Vector3::new(0.5, 0.5, 0.5));
            scene.push(object);
        }

        let view_dim = renderer.dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut camera  = Camera2d::new(logger,view_dim.x,view_dim.y);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        // We move the Camera 29 units away from the center.
        camera.set_position(Vector3::new(0.0, 0.0, 29.0));

        b.iter(move || {
            make_sphere(&mut scene, &performance);
            renderer.render(&mut camera);
        })
    }
}
