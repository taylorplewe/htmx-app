pub struct City {
    pub id: u32,
    pub name: String,
    pub state: String,
    pub country: String,
    pub sister_city_id: Option<u32>,
}

impl City {
    pub fn new(id: u32, name: String, state: String, country: String, sister_city_id: Option<u32>) -> Self {
        Self { id, name, state, country, sister_city_id }
    }
}