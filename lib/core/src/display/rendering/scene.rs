use crate::system::web::{get_element_by_id, dyn_into, Result};
use web_sys::HtmlElement;
use nalgebra::Vector2;

/// A collection for holding 3D `Object`s.
pub struct Scene {
    pub container : HtmlElement,
}

impl Scene {
    /// Searches for a HtmlElement identified by id and appends to it.
    ///
    /// # Arguments
    /// * id - the HtmlElement container's id
    pub fn new(id: &str) -> Result<Self> {
        let container = dyn_into(get_element_by_id(id)?)?;
        Ok(Self { container })
    }

    /// Gets the HtmlElement container's dimensions.
    pub fn get_dimensions(&self) -> Vector2<f32> {
        Vector2::new(self.container.client_width()  as f32,
                     self.container.client_height() as f32)
    }
}
