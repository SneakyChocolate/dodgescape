pub trait StringOperations {
    fn substring(&self, start: usize, end: usize) -> String;
}

impl StringOperations for String {
    fn substring(&self, start: usize, end: usize) -> String {
        let mut subs = "".to_owned();
        for i in start..=end {
            let char = self.chars().nth(i).unwrap();
            subs.push(char);
        }
        subs
    }
}

#[cfg(test)]
mod string_tests {
    use crate::string::*;
    #[test]
    fn substring() {
        let string = "hello my name is jonsa".to_owned();
        let r = string.substring(2, 5);
        let e = "llo ";
        assert_eq!(r, e);
    }
}

