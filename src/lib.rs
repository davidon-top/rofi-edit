pub mod item;
pub mod mode_logic;

use std::io::Read;

use serde_json::Value;

use crate::item::*;

struct Mode<'rofi> {
	api: rofi_mode::Api<'rofi>,
	entries: Vec<String>,
	items: Vec<ItemContainer>,
	state: State,
	message: String,
	output_singleobj: bool,
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
		let args = std::env::args().collect::<Vec<_>>();

		if args.contains(&"--edit-help".to_string()) {
			eprintln!(
				r#"
--edit-stdin	Read input from stdin, reads json until 2 newlines
--edit-file	<file>	Read input from file, json file
--edit-input	<input> Read input, should be a string containing json
--edit-example	Prints examples
--edit-out-singleobj	Outputs single object insted of array of objects see --edit-example for example
			"#
			);
			return Err(());
		}
		if args.contains(&"--edit-example".to_string()) {
			let its: Vec<ItemContainer> = vec![
				ItemContainer {
					name: "bool item".to_string(),
					item: Item::Bool { value: false },
				},
				ItemContainer {
					name: "int item".to_string(),
					item: Item::Int {
						value: 0,
						min: Some(0),
						max: None,
					},
				},
				ItemContainer {
					name: "float item".to_string(),
					item: Item::Float {
						value: 0.0,
						min: None,
						max: Some(69.0),
					},
				},
				ItemContainer {
					name: "string item".to_string(),
					item: Item::String {
						value: "hello".to_string(),
					},
				},
				ItemContainer {
					name: "enum items".to_string(),
					item: Item::Enum {
						value: 0,
						options: vec!["opt1".into(), "opt2".into(), "other_opt".into()],
					},
				},
			];
			eprintln!("Example input:\n{}\n\nOutput is the same with changed value keys\nKeys with value of null can be omited\n\n", serde_json::to_string(&its).unwrap());
			// single obj
			let mut val = serde_json::from_str::<Value>("{}").unwrap();
			let value  = val.as_object_mut().unwrap();
			its.iter().for_each(|item| {
				value.insert(item.name.clone(), serde_json::to_value(&item.item).unwrap());
			});
			eprintln!("Example when using --edit-singleobj\n\n {}", serde_json::to_string(&val).unwrap());

			return Err(());
		}
		let items: Vec<ItemContainer> = if args.contains(&"--edit-stdin".to_string()) {
			let res = read_stdin();
			serde_json::from_str(&res).unwrap()
		} else if let Some(file_pos) = args.iter().position(|arg| arg == "--edit-file") {
			let fp = args[file_pos + 1].clone();
			let res = std::fs::read_to_string(fp).unwrap();
			serde_json::from_str(&res).unwrap()
		} else if let Some(input_pos) = args.iter().position(|arg| arg == "--edit-input") {
			let res = args[input_pos + 1].clone();
			serde_json::from_str(&res).unwrap()
		} else {
			eprintln!("No input specified, use --edit-help for help");
			return Err(());
		};
		let out_type = args.contains(&"--edit-out-singleobj".to_string());
		let mut s = Self {
			api,
			entries: Vec::with_capacity(items.len() + 1),
			items,
			state: State::Main,
			message: String::new(),
			output_singleobj: out_type,
		};
		s.entries_from_items();
		Ok(s)
	}

	fn entries(&mut self) -> usize { self.entries.len() }

	fn entry_content(&self, line: usize) -> rofi_mode::String { self.entries[line].clone().into() }

	fn react(
		&mut self,
		event: rofi_mode::Event,
		input: &mut rofi_mode::String,
	) -> rofi_mode::Action {
		match event {
			rofi_mode::Event::Cancel { .. } => {
				if let State::Editing(_item) = self.state {
					*input = "".into();
					self.entries_from_items();
				} else {
					self.print_items();
					return rofi_mode::Action::Exit;
				}
			},
			rofi_mode::Event::Ok { selected, .. } => {
				if let State::Editing(_item) = self.state {
					self.finish_edit(Some(selected), None);
				} else if selected == self.entries() - 1 {
					self.print_items();
					return rofi_mode::Action::Exit;
				} else {
					self.enter_edit(selected);
				}
			},
			rofi_mode::Event::CustomInput { .. } => {
				if let State::Editing(_item) = self.state {
					self.finish_edit(None, Some(input));
					*input = "".into();
				} else {
					*input = rofi_mode::format!("");
				}
			},
			rofi_mode::Event::Complete {
				selected: Some(item),
			} => {
				*input = rofi_mode::format!("{}", self.entries[item]);
			},
			_ => {},
		}
		rofi_mode::Action::Reload
	}

	fn matches(&self, line: usize, matcher: rofi_mode::Matcher<'_>) -> bool {
		matcher.matches(&self.entries[line])
	}

	fn message(&mut self) -> rofi_mode::String { self.message.clone().into() }
}

rofi_mode::export_mode!(Mode<'_>);
