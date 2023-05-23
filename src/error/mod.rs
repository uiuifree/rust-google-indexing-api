use std::fmt::{Debug, Formatter};

pub enum GoogleApiError {
    Connection(String),
    JsonParse(String),
}

impl GoogleApiError {
    pub fn to_string(&self) -> String {
        match self {
            GoogleApiError::Connection(e) => { e.to_string() }
            GoogleApiError::JsonParse(e) => { e.to_string() }
        }
    }
}

impl Debug for GoogleApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}
