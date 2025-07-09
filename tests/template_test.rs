#[cfg(test)]
mod tests {
    // Use super::*; -> this imports parent modules

    #[test]
    fn example_test() {
        // Setup
        let a = 2;
        let b = 3;

        // Execute
        let result = a + b;

        // Assert
        assert_eq!(result, 5, "2 + 3 should equal 5");
    }
}