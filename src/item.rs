#[derive(serde::Deserialize, serde::Serialize)]
pub struct ItemContainer {
	pub name: String,
	pub item: Item,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Item {
	Bool {
		#[serde(default)]
		value: bool,
	},
	Int {
		#[serde(default)]
		value: i64,
		min: Option<i64>,
		max: Option<i64>,
	},
	Float {
		#[serde(default)]
		value: f64,
		min: Option<f64>,
		max: Option<f64>,
	},
	String {
		#[serde(default)]
		value: String,
	},
	Enum {
		#[serde(default)]
		value: usize,
		options: Vec<String>,
	},
}

impl Item {
	pub fn get_value(&self) -> String {
		match self {
			Item::Bool { value } => value.to_string(),
			Item::Int { value, .. } => value.to_string(),
			Item::Float { value, .. } => value.to_string(),
			Item::String { value } => value.clone(),
			Item::Enum { value, options } => options[*value].clone(),
		}
	}
}

pub enum State {
	Main,
	Editing(usize),
}
