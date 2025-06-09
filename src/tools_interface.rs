use create::openai::Tool;

trait Tool_Instance {
    // runs the function with its parameters given as Vector of Strings
    fn run(&self, params: Vec<String>)-> Option(String);
    // returns the definition and description of the function as defined in openai::Tool
    fn return_choice()-> openai::Tool;
}