use std::io::Read;

#[derive(serde::Deserialize, serde::Serialize)]
struct ItemContainer {
	name: String,
	item: Item,
}

#[derive(serde::Deserialize, serde::Serialize)]
enum Item {
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
	}
}

impl Item {
	fn get_value(&self) -> String {
		match self {
			Item::Bool { value } => value.to_string(),
			Item::Int { value, .. } => value.to_string(),
			Item::Float { value, .. } => value.to_string(),
			Item::String { value } => value.clone(),
			Item::Enum { value, options } => options[*value].clone(),
		}
	}
}

enum State {
	Main,
	Editing(usize),
}

struct Mode<'rofi> {
	api: rofi_mode::Api<'rofi>,
	entries: Vec<String>,
	items: Vec<ItemContainer>,
	state: State,
	message: String,
}

impl<'rofi> Mode<'rofi> {
	fn entries_from_items(&mut self) {
		self.state = State::Main;
		self.api.set_display_name("edit");
		self.entries.clear();
		self.items.iter().for_each(|item| {
			self.entries.push(format!("{}: {}", item.name, item.item.get_value()));
		});
		self.entries.push("Apply".to_string());
	}

	fn enter_edit(&mut self, item: usize) -> String {
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
				let min = min.map(|min| format!("Min: {};", min)).unwrap_or("".to_string());
				let max = max.map(|max| format!("Max: {};", max)).unwrap_or("".to_string());
				self.message = format!("Old value: {};\r{} {}", value, min, max);
			},
			Item::Float { value, min, max } => {
				let min = min.map(|min| format!("Min: {};", min)).unwrap_or("".to_string());
				let max = max.map(|max| format!("Max: {};", max)).unwrap_or("".to_string());
				self.message = format!("Old value: {};\r{} {}", value, min, max);
			},
			Item::String { .. } => {},
			Item::Enum { options, .. } => {
				options.iter().for_each(|option| self.entries.push(option.clone()));
			},
		}
		self.entries.push("Cancel".to_string());
		return item.item.get_value();
	}

	fn finish_edit(&mut self, selected: Option<usize>, custom_input: Option<&mut rofi_mode::String>) {
		match self.state {
			State::Main => {panic!("finish_edit called in main state")},
			State::Editing(item) => {
				let item = &mut self.items[item];
				match item.item {
					Item::Bool { .. } => {
						match selected {
							Some(0) => item.item = Item::Bool { value: true },
							Some(1) => item.item = Item::Bool { value: false },
							_ => {},
						}
					},
					Item::Int { value, min, max } => {
						let mut new_value = value.clone();
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
						item.item = Item::Int { value: new_value, min, max };
					},
					Item::Float { value, min, max } => {
						let mut new_value = value.clone();
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
						item.item = Item::Float { value: new_value, min, max };
					},
					Item::String { .. } => {
						if let Some(input) = custom_input {
							item.item = Item::String { value: input.into() };
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

	fn print_items(&self) {
		println!("{}", serde_json::to_string(&self.items).unwrap());
	}
}

// reads until 2 subsequent newlines
fn read_stdin() -> String {
	let mut inp = Vec::new();
	let mut buf = [0u8; 1];
	let mut handle = std::io::stdin().lock();
	loop {
		handle.read_exact(&mut buf).unwrap();
		inp.push(buf[0]);
		if inp.ends_with(b"\n\n") {
			break;
		}
	}
	String::from_utf8(inp).unwrap()
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
	const NAME: &'static str = "edit\0";

	fn init(mut api: rofi_mode::Api<'rofi>) -> Result<Self, ()> {
		api.set_display_name("edit");
		let args = std::env::args().into_iter().collect::<Vec<_>>();


		if args.contains(&"--edit-help".to_string()) {
			eprintln!(r#"
--edit-stdin	Read input from stdin, reads json until 2 newlines
--edit-file	<file>	Read input from file, json file
--edit-input	<input> Read input, should be a string containing json
--edit-example	Prints examples
			"#);
			return Err(());
		}
		if args.contains(&"--edit-example".to_string()) {
			let mut its: Vec<ItemContainer> = Vec::new();
			its.push(ItemContainer { name: "bool item".to_string(), item: Item::Bool { value: false } });
			its.push(ItemContainer { name: "int item".to_string(), item: Item::Int { value: 0, min: Some(0), max: None } });
			its.push(ItemContainer { name: "float item".to_string(), item: Item::Float { value: 0.0, min: None, max: Some(69.0) } });
			its.push(ItemContainer { name: "string item".to_string(), item: Item::String { value: "hello".to_string() } });
			its.push(ItemContainer { name: "enum items".to_string(), item: Item::Enum { value: 0, options: vec!["opt1".into(), "opt2".into(), "other_opt".into()] } });
			eprintln!("Example input:\n{}\n\nOutput is the same with changed value keys\nKeys with value of null can be omited", serde_json::to_string_pretty(&its).unwrap());
			return Err(());
		}
		let items: Vec<ItemContainer> = if args.contains(&"--edit-stdin".to_string()) {
			let res = read_stdin();
			serde_json::from_str(&res).unwrap()
		} else if let Some(file_pos) = args.iter().position(|arg| *arg == "--edit-file".to_string()) {
			let fp = args[file_pos + 1].clone();
			let res = std::fs::read_to_string(fp).unwrap();
			serde_json::from_str(&res).unwrap()
		} else if let Some(input_pos) = args.iter().position(|arg| *arg == "--edit-input".to_string()) {
			let res = args[input_pos + 1].clone();
			serde_json::from_str(&res).unwrap()
		} else {
			eprintln!("No input specified, use --edit-help for help");
			return Err(());
		};
		let mut s = Self {
			api,
			entries: Vec::with_capacity(items.len() + 1),
			items,
			state: State::Main,
			message: String::new(),
		};
		s.entries_from_items();
		Ok(s)
	}

	fn entries(&mut self) -> usize {
		self.entries.len()
	}

	fn entry_content(&self, line: usize) -> rofi_mode::String {
		self.entries[line].clone().into()
	}

	fn react(&mut self, event: rofi_mode::Event, input: &mut rofi_mode::String) -> rofi_mode::Action {
		match event {
			rofi_mode::Event::Cancel { .. } => {
				if let State::Editing(item) = self.state {
					*input = "".into();
					self.entries_from_items();
				} else {
					self.print_items();
					return rofi_mode::Action::Exit;
				}
			},
			rofi_mode::Event::Ok { selected, .. } => {
				if let State::Editing(item) = self.state {
					self.finish_edit(Some(selected), None);
				} else {
					if selected == self.entries() - 1 {
						self.print_items();
						return rofi_mode::Action::Exit;
					} else {
						self.enter_edit(selected);
					}
				}
			},
			rofi_mode::Event::CustomInput { selected, .. } => {
				if let State::Editing(item) = self.state {
					self.finish_edit(None, Some(input));
					*input = "".into();
				} else {
					*input = rofi_mode::format!("");
				}
			},
			rofi_mode::Event::Complete { selected } => {
				match selected {
					None => {},
					Some(item) => {
						*input = rofi_mode::format!("{}", self.entries[item]);
					}
				}
			},
			rofi_mode::Event::DeleteEntry { selected } => {},
			rofi_mode::Event::CustomCommand { number, selected } => {},
		}
		rofi_mode::Action::Reload
	}

	fn matches(&self, line: usize, matcher: rofi_mode::Matcher<'_>) -> bool {
		matcher.matches(&self.entries[line])
	}

	fn message(&mut self) -> rofi_mode::String {
	    self.message.clone().into()
	}
}

rofi_mode::export_mode!(Mode<'_>);
