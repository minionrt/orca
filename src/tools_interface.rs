#![allow(dead_code)]

use crate::openai;
use serde::{Deserialize, Serialize};

pub trait ToolInstance {
    /// Input argument type, this should be a struct which can be parsed from the received JSON
    type Args: for<'a> Deserialize<'a>;

    /// Output type
    type Out: Serialize;

    /// Runs the function
    fn run(input: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>>;

    /// Returns the definition and description of the function as defined in openai::Tool
    fn return_choice() -> openai::Tool;
}
