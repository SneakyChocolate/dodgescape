
use std::hash::RandomState;

use rand::Rng;

use crate::string::substring;


#[derive(Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r,g,b,a }
    }
    pub fn to_string(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }
    pub fn random() -> Self {
        let (r,g,b): (f32, f32, f32) = (
            rand::thread_rng().gen_range(0.0..=255.0),
            rand::thread_rng().gen_range(0.0..=255.0),
            rand::thread_rng().gen_range(0.0..=255.0)
        );
        Self::new(r,g,b,1.0)
    }
    pub fn from_str(color: &str) -> Self {
        let color = color.replace(" ", "");
        let start = color.find("(").unwrap() + 1;
        let end = color.find(")").unwrap() - 1;
        let rgba = substring(&color, start, end);
        let split = rgba.split(",").collect::<Vec<&str>>();
        let rgban = split.iter().map(|e| {
            match e.parse::<f32>() {
                Ok(v) => v,
                Err(_) => {
                    println!("{:?}", e);
                    0.0
                },
            }
        }).collect::<Vec<f32>>();
        let (r,g,b) = (rgban[0], rgban[1], rgban[2]);
        let a = if rgban.len() > 3 {
            rgban[3]
        }
        else {
            1.0
        };
        Self::new(r, g, b, a)
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r &&
        self.g == other.g &&
        self.b == other.b &&
        self.a == other.a
    }
}

#[cfg(test)]
mod color_test {
    use crate::color::*;
    #[test]
    fn fromstring() {
        let str = "rgb(3,62,21)";
        let result = Color::from_str(str);
        let exp = Color::new(3.0, 62.0, 21.0, 1.0);
        assert_eq!(result, exp);
    }
    #[test]
    fn fromstringalpha() {
        let str = " rgba(93,  62,21, 0.5)";
        let result = Color::from_str(str);
        let exp = Color::new(93.0, 62.0, 21.0, 0.5);
        assert_eq!(result, exp);
    }
}
