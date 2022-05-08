extern crate rusqlite;
extern crate anyhow;

use rusqlite::{Connection, Error};
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;
use time::Date;

use crate::util::*;

pub struct DbHandler {
	conn: Connection,
	commands: HashMap<&'static str, &'static str>
}

impl DbHandler {
	pub fn new(path: PathBuf) -> Self {
		let conn = Connection::open(path).unwrap();
		let mut commands = HashMap::new();
		commands.insert("test", "test result");
		commands.insert("insert field column", "ALTER TABLE entries ADD COLUMN :name :type;");
		commands.insert("insert field entry", "INSERT INTO fields (name, category, type, active) VALUES (':name', ':category', ':type', true);");
		commands.insert("insert entry", "INSERT INTO entries (date, ENTRY_COLUMNS) VALUES (ENTRY_VALUES);");
		commands.insert("get fields", "SELECT name, category, type, active FROM fields;");
		commands.insert("get active fields", "SELECT name, category, type, active FROM fields WHERE active = true;");
		commands.insert("get entries", "SELECT date, ENTRY_COLUMNS, tags FROM entries;");

		Self {
    		conn,
			commands
		}
	}

	pub fn initialize_db(path: PathBuf) -> Result<Self, Error> {
		let conn = Connection::open(path.clone());

		match conn {
			Ok(conn) => {
				conn.execute(
					"create table if not exists entries (
						id integer primary key,
						date integer not null,
						tags text
					)",
					[],
				)?;
		
				conn.execute(
					"create table if not exists fields (
						name text not null unique,
						category text not null,
						type text not null,
						active boolean not null
					)",
					[],
				)?;
		
				conn.execute(
					"create table if not exists recommendations (
						id integer primary key,
						type text not null,
						output text not null,
						input text not null,
						confidence float not null,
						hidden bool not null
					)",
					[],
				)?;

				conn.execute(
					"create table if not exists states (
						id integer primary key,
						name text not null,
						amount float,
						start_date text not null,
						end_date text
					)", 
					[],
				)?;

				conn.execute("INSERT INTO fields (name, category, type, active) VALUES ('tags', 'input', 'text', true);", [])?;
			},
			Err(err) => {
				println!("Error occurred when trying to create a database.");
				return Err(err)
			},
		}

		let dbh = DbHandler::new(path);

		Ok(dbh)
	}

	pub fn insert_field(&self, field: &Field) -> Result<()> {
		let (type_full, type_short) = match field.data_type {
			FieldType::Numeric => ("float", "n"),
			FieldType::Boolean => ("boolean", "b"),
			FieldType::Text => ("text", "t"),
		};
		let category = match field.category {
			FieldCategory::Input => "i",
			FieldCategory::Output => "o",
			FieldCategory::Hybrid => "h",
		};
		let ifc = self.commands.get("insert field column").unwrap().replace(":name", field.name.as_str()).replace(":type",type_full);
		let ife = self.commands.get("insert field entry").unwrap().replace(":name", field.name.as_str()).replace(":type",type_short).replace(":category", category);

		match self.conn.execute(&ifc, []) {
			Ok(_) => {
				match self.conn.execute(&ife, []) {
					Ok(_) => return Ok(()),
					Err(e) => return Err(anyhow::Error::new(e).context("Failed inserting row")),
				}
			},
			Err(e) => return Err(anyhow::Error::new(e).context("Failed inserting column")),
		}
	}

	//TODO: Change to get_active_fields?
	pub fn get_fields(&self) -> Result<Vec<Field>, Error> {
		let mut stmt = self.conn.prepare(self.commands.get("get active fields").unwrap())?;
		let fields = stmt.query_map([], |row| {
			let c1: String = row.get(1)?;
			let category = match c1.as_str() {
				"i" => FieldCategory::Input,
				"o" => FieldCategory::Output,
				"h" => FieldCategory::Hybrid,
				_ => FieldCategory::Input
			};

			let c2: String = row.get(2)?;
			let data_type = match c2.as_str() {
				"t" => FieldType::Text,
				"n" => FieldType::Numeric,
				"b" => FieldType::Boolean,
				_ => FieldType::Text,
			};

			Ok(Field {
				name: row.get(0)?,
				category,
				data_type,
				active: row.get(3)?,
			})
		})?;

		fields.collect()
	}

	pub fn get_entries(&self) -> Result<Vec<Entry>, Error> {
		let fields = self.get_fields()?;
		let ge = self.commands.get("get entries").unwrap();
		let field_columns: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
		let ge = ge.replace("ENTRY_COLUMNS", field_columns.join(", ").as_str());
		let mut stmt = self.conn.prepare(&ge)?;
		
		let entries = stmt.query_map([], |row| {
			let mut i = 0;
			let mut numeric_fields = HashMap::new();
			let mut boolean_fields = HashMap::new();
			let mut tags = Vec::new();

			println!("Before the loop");
			let date: i32 = row.get(i)?;
			i += 1;
			while i < 1+field_columns.len() {
				let field = fields.get(i-1).unwrap();
				println!("Loop iteration {}, looking at field {}", i, field.name);				
				match field.data_type {
					FieldType::Numeric => {
						let val: f64 = row.get(i)?;
						numeric_fields.insert(field.name.clone(), val);
					},
					FieldType::Boolean => {
						let val: bool = row.get(i)?;
						boolean_fields.insert(field.name.clone(), val);
					},
					FieldType::Text => {
						let tag_col: String = row.get(i)?;
						tags = tag_col.split(" ").map(|s| String::from(s)).collect();
					},
				}
				i += 1;
			}

			Ok(Entry {
				date: Date::from_julian_day(date).unwrap(),
				numeric_fields,
				boolean_fields,
				tags,
			})
		})?;

		entries.collect()
	}

	pub fn insert_entry(&self, entry: &Entry) -> Result<usize, Error> {
		println!("Inserting row");
		let ie = self.commands.get("insert entry").unwrap();
		let mut cols = String::new();
		let mut values = format!("{}, ", entry.date.to_julian_day());
		for (key, value) in &entry.boolean_fields {
			cols.push_str(format!("{}, ", key).as_str());
			values.push_str(format!("{}, ", value).as_str());
		}
		for (key, value) in &entry.numeric_fields {
			cols.push_str(format!("{}, ", key).as_str());
			values.push_str(format!("{}, ", value).as_str());
		}
		cols.push_str(format!("tags").as_str());
		values.push_str(format!("'{}'", entry.tags.join(" ")).as_str());

		let ie = ie.replace("ENTRY_COLUMNS", cols.as_str()).replace("ENTRY_VALUES", values.as_str());
		println!("Inserting row: {}", ie);
		self.conn.execute(&ie, [])
	}

}

#[cfg(test)]
pub mod db_tests {
    use std::{path::PathBuf, collections::HashMap};
	use std::fs;
    use time::{Date, Month};
    use crate::{db::DbHandler, util::*};

	pub fn setup_db(file: &str) -> DbHandler {
		// let p = format!("D:\\Programs\\mood\\{}.db", file);
        let path = PathBuf::from(file);
		if path.exists() {
			let _ = fs::remove_file(&path);
		}

		let dbh = DbHandler::initialize_db(PathBuf::from(path)).unwrap();

		dbh
	}

    #[test]
    fn field_creation() {
		let dbh = setup_db("test_field_creation.db");
		let field = Field {
			name: String::from("mood"),
			category: crate::util::FieldCategory::Output,
			data_type: crate::util::FieldType::Numeric,
			active: true,
		};
		
		let r = dbh.insert_field(&field);
		// println!("{:?}", r);
		assert!(r.is_ok());
		let vf = dbh.get_fields().unwrap();
		assert!(vf.len() == 2);
		assert!(field.eq(vf.get(1).unwrap()));
    }

	#[test]
	fn entry_insertion() {
		let dbh = setup_db("test_entry_insertion.db");

		let entry = Entry {
			date: Date::from_calendar_date(2022, Month::May, 12).unwrap(),
			numeric_fields: HashMap::new(),
			boolean_fields: HashMap::new(),
			tags: vec![String::from("argument::doug")],
		};
		let result = dbh.insert_entry(&entry);
		assert!(result.is_ok());
		assert!(result.unwrap() == 1);

		let returned = dbh.get_entries();
		println!("{:?}", returned);
		assert!(returned.is_ok());
		assert!(returned.unwrap().pop().unwrap() == entry);
	}
}