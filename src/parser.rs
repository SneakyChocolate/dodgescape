
pub fn get_variable(string: &String, name: &str) -> Option<String> {
    let declaration = format!("let {}", name);
    let var_start = match string.find(declaration.as_str()) {
        Some(r) => r,
        None => {return None;},
    };
    let rest = string[var_start..].to_owned();
    let value_start = match rest.find("= ") {
        Some(r) => r + 2,
        None => {return None;},
    };
    let value_end = match rest.find(";") {
        Some(r) => r,
        None => {return None;},
    };
    let value = Some(rest[value_start..value_end].to_owned());
    value
}

pub fn get_mouse(string: &String) -> Option<(i32, i32)> {
    let x_string = match get_variable(&string, "x") {
        Some(e) => e,
        None => {return None;},
    };
    let y_string = match get_variable(&string, "y") {
        Some(e) => e,
        None => {return None;},
    };

    let x: i32 = x_string.parse().unwrap();
    let y: i32 = y_string.parse().unwrap();

    Some((x, y))
}

pub fn get_keys_down(string: &String) -> Vec<String> {
    let value = get_variable(&string, "keys_down").unwrap();
    let keys_down = split(&value, ",");
    // println!("{:?}", keys_down);
    keys_down
}

pub fn level_map(string: &String) -> Vec<usize> {
    let mut map = vec![];
    //  bracket level
    let mut level: usize = 0;
    // string mode
    let mut mode: Option<char> = None;
    for c in string.chars() {
        match mode {
            Some(m) => {
                if c == m {
                    mode = None;
                }
            },
            None => {
                match c {
                    '"' => {mode = Some('"');},
                    '\'' => {mode = Some('\'');},
                    '(' => {level += 1;},
                    '[' => {level += 1;},
                    '{' => {level += 1;},
                    ')' => {level -= 1;},
                    ']' => {level -= 1;},
                    '}' => {level -= 1;},
                    _ => {},
                }
            },
        }
        map.push(level);
    }
    map
}

pub fn find_on_level(string: &String, level: usize, search: &str) -> Option<usize> {
    let map = level_map(&string);
    let mut start: usize = 0;

    while start < string.len() {
        let substring = string[start..].to_owned();
        match substring.find(search) {
            Some(i) => {
                if map[i] == level {
                    return Some(i);
                }
                else {
                    start = i + 1;
                }
            },
            None => return None,
        };
    }

    None
}

pub fn split(string: &String, seperator: &str) -> Vec<String> {
    let levels = level_map(&string);
    let mut strings = vec![];
    let mut next: usize = 0;
    while next < string.len() {
        let end = match string[next..].find(seperator) {
            Some(index) => index + next,
            None => string.len(),
        };

        strings.push(string[next..end].to_owned());

        next = end + seperator.len();
    }
    strings
}

pub fn split_level(string: &String, seperator: &str) -> Vec<String> {
    let levels = level_map(&string);
    let mut strings = vec![];
    let mut next: usize = 0;
    while next < string.len() {
        let s = string[next..].to_owned();
        let end = match find_on_level(&s, 0, seperator) {
            Some(index) => index + next,
            None => string.len(),
        };

        strings.push(string[next..end].to_owned());

        next = end + seperator.len();
    }
    strings
}

#[cfg(test)]
mod test_parser {
    use crate::parser::*;

    #[test]
    fn get_variable_test() {
        let expected = "4325".to_owned();
        let test_string = "let x: i32 = 4325;".to_owned();
        let result = get_variable(&test_string, "x").unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_test() {
        let expected = (3, 52);
        let test_string = "let x: i32 = 3;let y: i32 = 52;".to_owned();
        let result = get_mouse(&test_string).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn split_test() {
        let string = "i,want,some(a,b,c),milk".to_owned();
        let expected: Vec<String> = vec!["i".to_owned(), "want".to_owned(), "some(a".to_owned(), "b".to_owned(), "c)".to_owned(), "milk".to_owned()];
        let result = split(&string, ",");
        assert_eq!(expected, result);
    }

    #[test]
    fn level_map_test() {
        let string = "i,want,some(a,b,c),milk".to_owned();
        let expected: Vec<usize> = vec![0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0,0];
        let result = level_map(&string);
        assert_eq!(expected, result);
    }

    // TODO

    // #[test]
    // fn split_level_test() {
    //     let string = "i,want,some(a,b,c),milk".to_owned();
    //     let expected: Vec<String> = vec!["i".to_owned(), "want".to_owned(), "some(a,b,c)".to_owned(), "milk".to_owned()];
    //     let result = split_level(&string, ",");
    //     assert_eq!(expected, result);
    // }

    // #[test]
    // fn find_on_level_test() {
    //     let string = "i,want,some(a,b,c),milk".to_owned();
    //     assert_eq!(find_on_level(&string, 1, "a").unwrap(), 12);
    //     assert_eq!(find_on_level(&string, 0, "a").unwrap(), 3);
    // }
}

