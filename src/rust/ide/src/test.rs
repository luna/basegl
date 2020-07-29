//! Module for support code for writing tests.

use crate::prelude::*;

/// Utilities for mocking IDE components.
#[cfg(test)]
pub mod mock {
    use super::*;


    /// Data used to create mock IDE components.
    ///
    /// Contains a number of constants and functions building more complex structures from them.
    /// The purpose is to allow different parts of tests that mock different models using
    /// consistent data.
    #[allow(missing_docs)]
    pub mod data {
        use enso_protocol::language_server::Position;

        pub const PROJECT_NAME    : &str     = "MockProject";
        pub const MODULE_NAME     : &str     = "Mock_Module";
        pub const CODE            : &str     = "main = \n    2 + 2";
        pub const DEFINITION_NAME : &str     = "main";
        pub const TYPE_NAME       : &str     = "Mock_Type";
        pub const MAIN_FINISH     : Position = Position {line:1, character:9};

        pub fn module_path() -> crate::model::module::Path {
            crate::model::module::Path::from_mock_module_name(MODULE_NAME)
        }

        pub fn definition_name() -> crate::double_representation::definition::DefinitionName {
            crate::double_representation::definition::DefinitionName::new_plain(DEFINITION_NAME)
        }

        pub fn graph_id() -> crate::double_representation::graph::Id {
            crate::double_representation::graph::Id::new_plain_name(DEFINITION_NAME)
        }
    }

    pub fn indent(line:impl AsRef<str>) -> String {
        iformat!("    {line.as_ref()}")
    }

    pub fn main_from_lines(lines:impl IntoIterator<Item:AsRef<str>>) -> String {
        let body = lines.into_iter().map(indent).join("\n");
        iformat!("main = \n{body}")
    }
}
