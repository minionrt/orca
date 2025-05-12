#[cfg(test)]
mod tests {
    //use super::*; -> this imports parent modules

    #[test]
    fn example_test() {
        // setup 
        let a = 2;
        let b = 3;

        // execute 
        let result = a + b;

        // assert
        assert_eq!(result, 5, "2 + 3 should equal 5");
        assert!(true)
    }
}