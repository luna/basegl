//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]

use web_test::web_configure;
web_configure!(run_in_browser);

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen(module = "/tests/bench_test.js")]
extern "C" {
    fn set_gradient_bg(
        dom           : &JsValue,
        red           : &JsValue,
        green         : &JsValue,
        blue          : &JsValue);
}

#[cfg(test)]
mod tests {
    use basegl::display::rendering::*;
    use basegl::system::web::StyleSetter;
    use basegl::system::web::get_performance;
    use web_test::*;
    use web_sys::Performance;

    #[web_test(no_container)]
    fn invalid_container() {
        let scene = HTMLScene::new("nonexistent_id");
        assert!(scene.is_err(), "nonexistent_id should not exist");
    }

    #[web_test]
    fn object_behind_camera() {
        let mut scene = HTMLScene::new("object_behind_camera")
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0, "Scene should be empty");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let mut object = HTMLObject::new("div").unwrap();
        object.set_position(0.0, 0.0, 0.0);
        object.element.set_property_or_panic("background-color", "black");
        object.set_dimensions(100.0, 100.0);
        scene.add(object);

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        // We move the Camera behind the object so we don't see it.
        camera.set_position(0.0, 0.0, -100.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    fn create_scene(dom_id : &str) -> HTMLScene {
        let mut scene = HTMLScene::new(dom_id)
                                  .expect("Failed to create HTMLScene");
        assert_eq!(scene.len(), 0);

        scene.dom.set_property_or_panic("background-color", "black");

        // Iterate over 3 axes.
        for axis in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
            // Creates 10 HTMLObjects per axis.
            for i in 0 .. 10 {
                let mut object = HTMLObject::new("div").unwrap();
                object.set_dimensions(1.0, 1.0);

                // Using axis for masking.
                // For instance, the axis (0, 1, 0) creates:
                // (x, y, z) = (0, 0, 0) .. (0, 9, 0)
                let x = (i * axis.0) as f32;
                let y = (i * axis.1) as f32;
                let z = (i * axis.2) as f32;
                object.set_position(x, y, z);

                // Creates a gradient color based on the axis.
                let r = (x * 25.5) as u8;
                let g = (y * 25.5) as u8;
                let b = (z * 25.5) as u8;
                let color = format!("rgba({}, {}, {}, {})", r, g, b, 1.0);

                object.element.set_property_or_panic("background-color", color);
                scene.add(object);
            }
        }
        assert_eq!(scene.len(), 30, "We should have 30 HTMLObjects");
        scene
    }

    #[web_test]
    fn rhs_coordinates() {
        let scene = create_scene("rhs_coordinates");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);

        // We move the Camera 29 units away from the center.
        camera.set_position(0.0, 0.0, 29.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[web_test]
    fn rhs_coordinates_from_back() {
        use std::f32::consts::PI;
        let scene = create_scene("rhs_coordinates_from_back");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);

        // We move the Camera -29 units away from the center.
        camera.set_position(0.0, 0.0, -29.0);
        // We rotate it 180 degrees so we can see the center of the scene
        // from behind.
        camera.set_rotation(0.0, PI, 0.0);

        let renderer = HTMLRenderer::new();
        renderer.render(&mut camera, &scene);
    }

    #[web_bench]
    fn camera_movement(b: &mut Bencher) {
        let scene = create_scene("camera_movement");

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        b.iter(move || {
            let t = (performance.now() / 1000.0) as f32;
            // We move the Camera 29 units away from the center.
            camera.set_position(t.sin() * 5.0, t.cos() * 5.0, 29.0);

            let renderer = HTMLRenderer::new();
            renderer.render(&mut camera, &scene);
        })
    }

    fn make_sphere(scene : &mut HTMLScene, performance : &Performance) {
        use super::set_gradient_bg;

        let t = (performance.now() / 1000.0) as f32;
        let length = scene.len() as f32;
        for (i, object) in scene.into_iter().enumerate() {
            let i = i as f32;
            let d = (i / length - 0.5) * 2.0;

            let mut y = d;
            let r = (1.0 - y * y).sqrt();
            let mut x = (y * 100.0 + t).cos() * r;
            let mut z = (y * 100.0 + t).sin() * r;

            x += (y * 1.25 + t * 2.50).cos() * 0.5;
            y += (z * 1.25 + t * 2.00).cos() * 0.5;
            z += (x * 1.25 + t * 3.25).cos() * 0.5;
            object.set_position(x * 5.0, y * 5.0, z * 5.0);

            let faster_t = t * 100.0;
            let r = (i +   0.0 + faster_t) as u8 % 255;
            let g = (i +  85.0 + faster_t) as u8 % 255;
            let b = (i + 170.0 + faster_t) as u8 % 255;
            set_gradient_bg(&object.element, &r.into(), &g.into(), &b.into());
        }
    }

    #[web_bench]
    fn object_x1000(b: &mut Bencher) {
        let mut scene = HTMLScene::new("object_x1000")
                                  .expect("Failed to create scene");
        scene.dom.set_property_or_panic("background-color", "black");

        for _ in 0..1000 {
            let mut object = HTMLObject::new("div")
                                    .expect("Failed to create object");
            object.set_dimensions(1.0, 1.0);
            object.set_scale(0.5, 0.5, 0.5);
            scene.add(object);
        }

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        // We move the Camera 29 units away from the center.
        camera.set_position(0.0, 0.0, 29.0);

        let renderer = HTMLRenderer::new();
        make_sphere(&mut scene, &performance);

        b.iter(move || {
            renderer.render(&mut camera, &scene);
        })
    }

    #[web_bench]
    fn object_x400_update(b: &mut Bencher) {
        let mut scene = HTMLScene::new("object_x400_update")
                                  .expect("Failed to create scene");
        scene.dom.set_property_or_panic("background-color", "black");

        for _ in 0..400 {
            let mut object = HTMLObject::new("div")
                                    .expect("Failed to create object");
            object.set_dimensions(1.0, 1.0);
            object.set_scale(0.5, 0.5, 0.5);
            scene.add(object);
        }

        let view_dim = scene.get_dimensions();
        assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));

        let aspect_ratio = view_dim.x / view_dim.y;
        let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
        let performance = get_performance()
                         .expect("Couldn't get performance obj");

        // We move the Camera 29 units away from the center.
        camera.set_position(0.0, 0.0, 29.0);

        let renderer = HTMLRenderer::new();


        b.iter(move || {
            make_sphere(&mut scene, &performance);
            renderer.render(&mut camera, &scene);
        })
    }
}
