use std::collections::HashMap;
use time::Date;

#[derive(PartialEq, Clone, Debug)]
pub struct Field {
	pub name: String,
	pub category: FieldCategory,
	pub data_type: FieldType,
	pub active: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Entry {
	pub date: Date,
	pub numeric_fields: HashMap<String, f64>,
	pub boolean_fields: HashMap<String, bool>,
	pub tags: Vec<String>,
}


#[derive(PartialEq, Clone, Debug, Default)]
pub struct Trend {
	pub start: u32,
	pub end: u32,
	pub orig_val: f64,
	pub slope: f64,
	pub stddev: f64,
	//TODO: include # of data points considered?
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldCategory {
	Input,
	Output,
	Hybrid
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
	Numeric,
	Boolean,
	Text
}

#[cfg(test)]
pub mod test_utils {
	use crate::prelude::*;
	use csv::Reader;
	use std::fs;

	pub fn setup_db(file: &str) -> DbHandler {
        let path = PathBuf::from(file);
		if path.exists() {
			let _ = fs::remove_file(&path);
		}

		let dbh = DbHandler::initialize_db(PathBuf::from(path)).unwrap();

		dbh
	}

	pub fn import_csv(dbh: &DbHandler, file: &str) {
		let mut reader = Reader::from_path(file).unwrap();
		let headers = reader.headers().unwrap();
		let mut header_cols = Vec::new();
		let mut fields = Vec::new();
		for record in headers {
			header_cols.push(String::from(&record[2..]));
			if record == "date" || record == "tags" {
				continue;
			}
			let category = match &record[0..1] {
				"I" => FieldCategory::Input,
				"O" => FieldCategory::Output,
				_ => FieldCategory::Hybrid,
			};
			let data_type = match &record[1..2] {
				"N" => FieldType::Numeric,
				"B" => FieldType::Boolean,
				_ => FieldType::Text
			};

			let field = Field {
				name: String::from(&record[2..]),
				category,
				data_type,
				active: true,
			};
			let _ = dbh.insert_field(&field);
			fields.push(field);
		}

		let field = Field {
			name: String::from("tags"),
			category: FieldCategory::Input,
			data_type: FieldType::Text,
			active: true,
		};
		// let _ = dbh.insert_field(&field);
		fields.push(field);
		// println!("{:#?}", fields);

		for record in reader.records() {
			// println!("Beginning row parsing");
			let record = record.unwrap();
			let mut date = None;
			let mut i = 0;
			let mut numeric_fields = HashMap::new();
			let mut boolean_fields = HashMap::new();
			let mut tags = Vec::new();

			for a in &record {
				// println!("{}", a);
				if i == 0 {
					let n: i32 = a.parse().unwrap();
					date = Some(Date::from_julian_day(n).unwrap());
				} else {
					let field = fields.get(i-1).unwrap();
					match field.data_type {
						FieldType::Numeric => { numeric_fields.insert(field.name.clone(), a.parse().unwrap()); },
						FieldType::Boolean => { boolean_fields.insert(field.name.clone(), a.parse().unwrap()); },
						FieldType::Text => { tags = a.split(" ").map(|s| String::from(s)).collect(); },
					}
				}
				// record.unwrap()
				i += 1;
			}

			let entry = Entry {
				date: date.unwrap(),
				numeric_fields,
				boolean_fields,
				tags,
			};
			let a = dbh.insert_entry(&entry);
			if let Err(message) = a {
				println!("Error occurred: {}", message);
			}
		}
	}
}