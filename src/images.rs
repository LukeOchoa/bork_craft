use serde_derive::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Imager {
    #[serde(rename = "Name")]
    pub name: String,
}

impl Imager {
    pub fn default() -> Self {
        Self {
            name: Default::default(),
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
