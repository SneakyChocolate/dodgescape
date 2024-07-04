
#[derive(Debug)]
pub struct Http_request {
    pub request_line: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<String>,
}

impl PartialEq for Http_request {
    fn eq(&self, other: &Self) -> bool {
        self.request_line == other.request_line &&
        self.headers == other.headers &&
        self.body == other.body
    }
}
impl Http_request {
    pub fn new() -> Http_request {
        Http_request {
            request_line: "".to_owned(),
            headers: vec![],
            body: vec![],
        }
    }
    pub fn parse_header(header: &String) -> (String, String) {
        let seperator = ": ";
        let seperator_index = header.find(seperator).unwrap();
        (header[..seperator_index].to_owned(), header[(seperator_index + seperator.len())..].to_owned())
    }
    pub fn parse(http_string: &String) -> Result<Http_request, String> {
        let lines: Vec<String> = http_string.lines().map(|line| line.to_owned()).collect();
        let request_line = match lines.first() {
            Some(e) => e,
            None => {return Err("no request line".to_owned());},
        };
        let mut headers: Vec<(String, String)> = vec![];
        let mut body = vec![];
        let mut writing_body = false;
        for line in &lines {
            if line == lines.first().unwrap() {
                continue;
            }
            if line.is_empty() {
                writing_body = true;
                continue;
            }
            if !writing_body {
                headers.push(Http_request::parse_header(line));
            }
            else {
                body.push(line.clone());
            }
        }

        let request = Http_request {
            request_line: request_line.clone(),
            headers: headers,
            body: body,
        };
        Ok(request)
    }
    pub fn get_header<'a>(&'a self, name: String) -> Option<&'a String> {
        let header_tuple = self.headers.iter().find(|(key, value)| {if *key == name {true} else {false}});
        match header_tuple {
            None => None,
            Some((key, value)) => Some(value),
        }
    }
}

#[cfg(test)]
mod http_tests {
    use crate::http::Http_request;

    #[test]
    fn string_to_lines_test() {
        let string = "reqline\r\nheader1\nheader2\r\n\r\nbody".to_owned();
        let lines: Vec<String> = string.lines().map(|line| line.to_owned()).collect();
        let expected = vec![
            "reqline".to_owned(),
            "header1".to_owned(),
            "header2".to_owned(),
            "".to_owned(),
            "body".to_owned(),
        ];
        assert_eq!(lines, expected);
    }
    #[test]
    fn parse_header_test() {
        let str = "Content-Length: 34".to_owned();
        let act = Http_request::parse_header(&str);
        let exp = ("Content-Length".to_owned(), "34".to_owned());
        assert_eq!(act, exp);
    }
    #[test]
    fn parse_test() {
        let str = "GET / HTTP/1.1\ncontent-type: json\ncontent-length: 342\r\n\r\nthis is a body".to_owned();
        let act = Http_request::parse(&str).unwrap();
        let exp = Http_request {
            request_line: "GET / HTTP/1.1".to_owned(),
            headers: vec![
                ("content-type".to_owned(), "json".to_owned()),
                ("content-length".to_owned(), "342".to_owned()),
            ],
            body: vec!["this is a body".to_owned()],
        };
        assert_eq!(act, exp);
    }
}
