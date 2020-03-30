#![cfg(target_arch = "wasm32")]

use crate::prelude::*;

use crate::api;

use api::Ast;
use ast::IdMap;

use wasm_bindgen::prelude::*;

pub type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Fail)]
pub enum Error {
    #[fail(display = "JSON (de)serialization failed: {:?}", _0)]
    JsonSerializationError(#[cause] serde_json::error::Error),

    #[fail(display = "Scala parser failed: {:?}.", _0)]
    ScalaException(String),
}

impl From<Error> for api::Error {
    fn from(e:Error) -> Self {
        api::interop_error(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error:serde_json::error::Error) -> Self {
        Error::JsonSerializationError(error)
    }
}

impl From<JsValue> for Error {
    fn from(jsvalue:JsValue) -> Self {
        Error::ScalaException(format!("{:?}", jsvalue))
    }
}

#[wasm_bindgen(module = "/pkg/scala-parser.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn parse
    (input:String, ids:String) -> std::result::Result<String,JsValue>;
    #[wasm_bindgen(catch)]
    fn parse_with_metadata
    (content:String) -> std::result::Result<String,JsValue>;
}

/// Wrapper over the JS-compiled parser.
///
/// Can only be used when targeting WebAssembly.
#[derive(Debug,Clone,Copy)]
pub struct Client {}

impl Client {
    pub fn new() -> Result<Client> {
        Ok(Client {})
    }

    pub fn parse(&self, program:String, ids:IdMap) -> api::Result<Ast> {
        let ast = || {
            let json_ids = serde_json::to_string(&ids)?;
            let json_ast = parse(program,json_ids)?;
            let      ast = serde_json::from_str(&json_ast)?;
            Result::Ok(ast)
        };
        Ok(ast()?)
    }

    pub fn parse_with_metadata<M:api::Metadata>
    (&self, program:String) -> api::Result<api::SourceFile<M>> {
        let result = || {
            let json   = &parse_with_metadata(program)?;
            let module = serde_json::from_str(&json)?;
            Result::Ok(module)
        };
        Ok(result()?)
    }
}
