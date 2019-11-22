use basegl::system::web::create_element;
use basegl::system::web::dyn_into;
use basegl::system::web::AttributeSetter;
use basegl::system::web::StyleSetter;
use basegl::system::web::NodeInserter;
use web_sys::HtmlElement;

use super::Group;

pub struct Container {
    pub div: HtmlElement,
    pub header: HtmlElement,
    pub container : HtmlElement
}

impl Container {
    pub fn new(group : &str, name: &str, width: f32, height: f32) -> Self {
        let div = create_element("div").expect("div");
        let div : HtmlElement = dyn_into(div).expect("HtmlElement");

        let width = format!("{}px", width);
        div.set_property_or_panic("width"   , &width);
        div.set_property_or_panic("height"  , format!("{}px", height + 17.0));
        div.set_property_or_panic("border"  , "1px solid black");
        div.set_property_or_panic("position", "relative");
        div.set_property_or_panic("margin"  , "10px");

        let header = create_element("center").expect("div");
        let header : HtmlElement = dyn_into(header).expect("HtmlElement");
        header.set_inner_html(&format!("{}", name));
        header.set_property_or_panic("width" , &width);
        header.set_property_or_panic("height", format!("{}px", 16.0));
        header.set_property_or_panic("border-bottom", "1px solid black");
        header.set_property_or_panic("position", "relative");

        div.append_child_or_panic(&header);

        let container = create_element("div").expect("div");
        let container : HtmlElement = dyn_into(container).expect("HtmlElement");

        container.set_property_or_panic("width" , width);
        container.set_property_or_panic("height", format!("{}px", height));
        container.set_attribute_or_panic("id", name);
        container.set_property_or_panic("position", "relative");

        div.append_child_or_panic(&container);

        Group::new(group).div.append_child_or_panic(&div);
        Self { div, header, container }
    }
}
