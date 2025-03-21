pub struct City {
    pub id: u32,
    pub name: String,
    pub country: String,
}

impl City {
    pub fn new(id: u32, name: String, country: String) -> Self {
        Self { id, name, country }
    }
}