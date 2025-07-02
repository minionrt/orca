#![allow(dead_code)]

use crate::openai;
use serde::{Deserialize, Serialize};

pub(crate) trait ToolInstance {
    type Args: for<'a> Deserialize<'a>;
    type Out: Serialize;
    // runs the function with its parameters given as Vector of Strings
    fn run(input: Self::Args) -> Result<Self::Out, Box<dyn std::error::Error>>;
    // returns the definition and description of the function as defined in openai::Tool
    fn return_choice() -> openai::Tool;
}
