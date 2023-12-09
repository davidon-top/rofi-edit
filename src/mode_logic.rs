use crate::{item::*, Mode};

impl<'rofi> Mode<'rofi> {
	pub fn entries_from_items(&mut self) {
		self.state = State::Main;
		self.api.set_display_name("edit");
		self.entries.clear();
		self.items.iter().for_each(|item| {
			self.entries
				.push(format!("{}: {}", item.name, item.item.get_value()));
		});
		self.entries.push("Apply".to_string());
	}

	pub fn enter_edit(&mut self, item: usize) -> String {
		self.state = State::Editing(item);
		let item = &self.items[item];
		self.api.set_display_name(format!("editing {}", item.name));
		self.message = format!("Old value: {}", item.item.get_value());
		self.entries.clear();
		match &item.item {
			Item::Bool { .. } => {
				self.entries.push("true".to_string());
				self.entries.push("false".to_string());
			},
			Item::Int { value, min, max } => {
				let min = min
					.map(|min| format!("Min: {};", min))
					.unwrap_or("".to_string());
				let max = max
					.map(|max| format!("Max: {};", max))
					.unwrap_or("".to_string());
				self.message = format!("Old value: {};\r{} {}", value, min, max);
			},
			Item::Float { value, min, max } => {
				let min = min
					.map(|min| format!("Min: {};", min))
					.unwrap_or("".to_string());
				let max = max
					.map(|max| format!("Max: {};", max))
					.unwrap_or("".to_string());
				self.message = format!("Old value: {};\r{} {}", value, min, max);
			},
			Item::String { .. } => {},
			Item::Enum { options, .. } => {
				options
					.iter()
					.for_each(|option| self.entries.push(option.clone()));
			},
		}
		self.entries.push("Cancel".to_string());
		item.item.get_value()
	}

	pub fn finish_edit(
		&mut self,
		selected: Option<usize>,
		custom_input: Option<&mut rofi_mode::String>,
	) {
		match self.state {
			State::Main => {
				panic!("finish_edit called in main state")
			},
			State::Editing(item) => {
				let item = &mut self.items[item];
				match item.item {
					Item::Bool { .. } => match selected {
						Some(0) => item.item = Item::Bool { value: true },
						Some(1) => item.item = Item::Bool { value: false },
						_ => {},
					},
					Item::Int { value, min, max } => {
						let mut new_value = value;
						if let Some(input) = custom_input {
							new_value = input.replace("\\-", "-").parse().unwrap();
						}
						if let Some(min) = min {
							if new_value < min {
								new_value = min;
							}
						}
						if let Some(max) = max {
							if new_value > max {
								new_value = max;
							}
						}
						item.item = Item::Int {
							value: new_value,
							min,
							max,
						};
					},
					Item::Float { value, min, max } => {
						let mut new_value = value;
						if let Some(input) = custom_input {
							new_value = input.replace("\\-", "-").parse().unwrap();
						}
						if let Some(min) = min {
							if new_value < min {
								new_value = min;
							}
						}
						if let Some(max) = max {
							if new_value > max {
								new_value = max;
							}
						}
						item.item = Item::Float {
							value: new_value,
							min,
							max,
						};
					},
					Item::String { .. } => {
						if let Some(input) = custom_input {
							item.item = Item::String {
								value: input.into(),
							};
						}
					},
					Item::Enum { ref mut value, .. } => {
						if let Some(selected) = selected {
							*value = selected;
						}
					},
				};
				self.entries_from_items();
			},
		}
	}

	pub fn print_items(&self) {
		println!("{}", serde_json::to_string(&self.items).unwrap());
	}
}
