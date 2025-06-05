#![allow(dead_code)]

//you can find all the openai models here: https://platform.openai.com/docs/models
const SMART_GPT_MODEL: &str = "o4-mini-2025-04-16";
const BASIC_GPT_MODEL: &str = "gpt-4.1-nano-2025-04-14";
const SMART_COHERE_MODEL: &str = "cohere/command-a-03-2025";
const BASIC_COHERE_MODEL: &str = "cohere/command-r";
const GEMINI: &str = "gemini-2.5-flash-preview-05-20";
const TEST_MODEL: &str = "test-model"; //no functionality (yet), just used for testing !
pub enum Model {
    Smart,
    Basic,
    Gemini,
    Test,
}

impl From<Model> for String {
    fn from(m: Model) -> String {
        match m {
            Model::Smart => SMART_COHERE_MODEL.to_string(),
            Model::Basic => BASIC_COHERE_MODEL.to_string(),
            Model::Gemini => GEMINI.to_string(),
            Model::Test => TEST_MODEL.to_string(),
        }
    }
}
