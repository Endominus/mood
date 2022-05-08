mod db;
mod util;

// use trendar;
use clap::{Parser, Subcommand};
use std::{io, path::PathBuf};

use crate::db::DbHandler;
use crate::util::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	#[clap(name = "init")]
	Initialize {
		#[clap(short, long)]
		database: Option<String>,
	},
	#[clap(name = "config")]
	Configure,
	Edit,
	Toggle { tag: Option<String> },
	Analyze,
}

fn main() {
    let cli = Cli::parse();
	match &cli.command {
		Some(command) => {
			match command {
    			Commands::Initialize { database } => init(database),
				Commands::Configure => { println!("Configuring something"); },
				Commands::Edit => println!("Editing an entry"),
				Commands::Toggle { tag } => match tag {
					Some(s) => println!("Toggling the {} state.", s),
					None => println!("Listing all states in database"),
    			},
				Commands::Analyze => println!("Performing analysis..."),
			}
		},
    	None => println!("Entering data"),
	}
	
}

fn init(database: &Option<String>) {
	// let config_directory = String::from("C:\\Users\\me\\Documents\\mood");
	println!("The program will initialize!");
	if let Some(database) = database {
		println!("Received database directory: {}", database);
		let dbh = DbHandler::initialize_db(PathBuf::from(database));
		match dbh {
			Ok(_) => println!("Database created successfully!"),
			Err(e) => println!("Something went wrong! {}", e),
		}
	}
	// println!("Detecting where the config file is.");
	// println!("Config file will be stored at {config_directory}. Please enter where you'd like the database to be stored [{config_directory}]:");

	let mut fields = Vec::new();
	println!("There are three categories of data that can be tracked; pure inputs, which are considered as causal factors\nto the other categories, pure outputs, which are never considered as inputs to any other data,\nand hybrids, which are treated as both inputs and outputs.\n");
	loop {
		let mut field_name = String::new();
		let mut confirm = String::new();
		println!("Please enter a new field name (leave empty to finish):");
		io::stdin().read_line(&mut field_name).expect("Invalid input received");
		if field_name.trim().is_empty() {
			break
		}

		let category = select_category();
		let field_type = select_type();
		
		print!("Name: {field_name}");
		println!("Category: {:?}", category);
		println!("Type: {:?}", field_type);
		println!("Is this data correct? [Y/n]:");
		io::stdin().read_line(&mut confirm).expect("Invalid input received");
		if confirm.trim().is_empty() || confirm.trim().to_lowercase() == String::from("y") {
			fields.push(
				(
					String::from(field_name.trim()), 
					category, 
					field_type,
				)
			);
		}
	}
	println!("Fields added are as below:");
	println!("{:#?}", fields);
}

fn select_category() -> FieldCategory {
	let mut field_category = String::new();
	loop {
		println!("Please enter the field category [(i)nput/(o)utput/(h)ybrid]:");
		io::stdin().read_line(&mut field_category).expect("Invalid input received");
		let category_option = match field_category.to_lowercase().trim() {
			"o" | "output" => Some(FieldCategory::Output),
			"i" | "input" => Some(FieldCategory::Input),
			"h" | "hybrid" => Some(FieldCategory::Hybrid),
			_ => None
		};
		if let Some(fc) = category_option {
			return fc
		} else {
			println!("Invalid choice. Please select one of the listed options.")
		}
	}
}

fn select_type() -> FieldType {
	let mut field_type = String::new();
	loop {
		println!("Please enter the data type [(n)umeric/(b)oolean]:");
		io::stdin().read_line(&mut field_type).expect("Invalid input received");
		let type_option = match field_type.to_lowercase().trim() {
			"n" | "numeric" => Some(FieldType::Numeric),
			"b" | "boolean" => Some(FieldType::Boolean),
			_ => None,
		};
		if let Some(ft) = type_option {
			return ft
		} else {
			println!("Invalid choice. Please select one of the listed options.")
		}
	}

}

// fn sub<T: Sub>(source: T, dest: T) -> <T as Sub>::Output {
// 	source - dest

// 	// println!("{:?}", source.sub(dest));
// }

// fn test() {
// 	let path = "D:\\Programs\\mood\\test1.db";
// 		let dbh = DbHandler::initialize_db(PathBuf::from(path)).unwrap();
// 		let field = Field {
// 			name: String::from("mood"),
// 			category: crate::util::FieldCategory::Output,
// 			data_type: crate::util::FieldType::Numeric,
// 			active: true,
// 		};
		
// 		let a = dbh.insert_field(field.clone());
// 		let vf = dbh.get_fields().unwrap();
// 		// assert!(vf.len() == 1);
// 		// assert!(field.eq(vf.get(0).unwrap()));
// }

#[cfg(test)]
mod main_tests {
	use std::collections::HashMap;

use crate::{db::{db_tests::setup_db, DbHandler}, util::{FieldCategory, FieldType, Field, Entry}};
	use csv::Reader;
	use time::Date;

	fn import_csv(dbh: &DbHandler, file: &str) {
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
			println!("Beginning row parsing");
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

	#[test]
	fn import_test() {
		let dbh = setup_db("test_import.db");
		import_csv(&dbh, "test_data.csv");
		
		let entries = dbh.get_entries();
		assert!(entries.is_ok(), "entries returned: {:?}", entries);
		let mut entries = entries.unwrap();
		assert!(entries.len() == 1);
		// debug_assert!()

		//date,ONmood,ONenergy,ONproductivity,HNsleep_quality,INcalories,IBexercise,tags
		//22124,3,2,4,6,1700,true,ate:gluten spoke:diego
		let entry = Entry {
			date: Date::from_julian_day(22124).unwrap(),
			numeric_fields: HashMap::from([
				(String::from("mood"), 3.0),
				(String::from("energy"), 2.0),
				(String::from("productivity"), 4.0),
				(String::from("sleep_quality"), 6.0),
				(String::from("calories"), 1700.0),
			]),
			boolean_fields: HashMap::from([(String::from("exercise"), true)]),
			tags: vec![String::from("ate:gluten"), String::from("spoke:diego")],
		};

		assert!(entries.pop().unwrap() == entry)

	}
}