#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_shared_function() {
        let t = trybuild::TestCases::new();
        t.pass("tests/shared_function.rs");
    }
}
