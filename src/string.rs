
pub fn substring(string: &String, start: usize, end: usize) -> String {
    let mut subs = "".to_owned();
    for i in start..=end {
        let char = string.chars().nth(i).unwrap();
        subs.push(char);
    }
    subs
}

