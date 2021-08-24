pub struct MongoConfig {
    pub connection: String,
    pub database: String
}

impl MongoConfig {
    pub fn default() -> Self {
        Self {
            connection: String::from("mongodb://localhost:27017"),
            database: String::from("lab_support_data")
        }
    }
}