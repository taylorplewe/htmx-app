pub struct Request {
    pub method: String,
    pub path: String,
}
impl Request {
    pub fn new(req_str: &str) -> Self {
        let mut split = req_str.split(' ');
        Self {
            method: String::from(split.next().unwrap()),
            path: String::from(split.next().unwrap()),
        }
    }
}