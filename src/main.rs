mod db;
mod util;
mod analysis;

mod prelude {
	pub use std::path::PathBuf;
	pub use crate::db::DbHandler;
	pub use crate::util::*;
	pub use crate::analysis::*;
	pub use std::collections::HashMap;
	pub use time::Date;
	pub use anyhow::{Result, anyhow};
	pub use dirs;
}

use prelude::*;
use core::f64;
use std::{io, fs, os};
// use std::;
// use trendar;
use clap::{Parser, Subcommand};

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
				Commands::Analyze => analyze_db(),
			}
		},
    	None => insert_entry(),
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

	// let mut config_dir = if cfg!(windows) {
	// 	PathBuf::from("~\\Documents\\mood\\")
	// } else {
	// 	PathBuf::from("~/.config/mood/")
	// };
	let config_dir = dirs::config_dir();
	if let Some(mut path) = config_dir {
		path.push("mood");
		if !path.exists() {
			if fs::create_dir_all(&path).is_ok() {
				path.push("config.toml");
				create_config(&path);
			} else {
				println!("Error occurred when creating config directory.");
				return
			}
		}
	} else {
		println!("Error occurred when retrieving user directory.");
		return
	}

	let mut db_dir = dirs::config_dir().unwrap();
	db_dir.push("mood");
	db_dir.push("mood.db");

	// TODO: Give option for changing DB location
	let dbh = if !db_dir.exists() {
		DbHandler::initialize_db(db_dir).unwrap()
	} else {
		DbHandler::new(db_dir)
	};
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
		field_name = field_name.replace(" ", "_");

		let category = select_category();
		let field_type = select_type();
		
		print!("Name: {field_name}");
		println!("Category: {:?}", category);
		println!("Type: {:?}", field_type);
		println!("Is this data correct? [Y/n]:");
		io::stdin().read_line(&mut confirm).expect("Invalid input received");
		if confirm.trim().is_empty() || confirm.trim().to_lowercase() == String::from("y") {
			fields.push(
				Field {
					name: String::from(field_name.trim()),
					category, 
					data_type: field_type,
					active: true,
				}
			);
		}
	}
	println!("Fields added are as below:");
	println!("{:#?}", fields);
	for field in fields {
		let _ = dbh.insert_field(&field);
	}
}

fn create_config(_config_directory: &PathBuf) {
	println!("This is where the config would be created!");
    // todo!()
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

fn insert_entry() {
	//TODO: Grab location from config file.
	let mut db_dir = dirs::config_dir().unwrap();
	db_dir.push("mood");
	db_dir.push("mood.db");

	let dbh = DbHandler::new(db_dir);
	let fields = dbh.get_fields().unwrap();
	let mut num_hm = HashMap::new();
	let mut bool_hm = HashMap::new();
	let mut tags = Vec::new();
	let date = Date::from(time::OffsetDateTime::now_utc().date());
	println!("Entering data for today, {}.", date);
	println!("If you do not wish to enter data, leave the field blank.");
	for field in fields {
		// println!("Back in the loop");
		match field.data_type {
			FieldType::Numeric => {
				let data = get_numeric_data(&field.name);
				if let Some(data) = data {
					num_hm.insert(field.name, data);
				}
			},
			FieldType::Boolean => {
				let data = get_boolean_data(&field.name);
				if let Some(data) = data {
					bool_hm.insert(field.name, data);
				}
			},
			FieldType::Text => {
				let mut data = String::new();
				println!("Please write down any notable tags for the day, separated by spaces.");
				io::stdin().read_line(&mut data).expect("Invalid input received");
				tags = data
					.split_ascii_whitespace()
					.filter(|s| s.len() > 0)
					.map(|s| String::from(s))
					.collect();
				// println!("{:?}", tags);
			},
		}
	}

	let entry = Entry {
		date,
		numeric_fields: num_hm,
		boolean_fields: bool_hm,
		tags,
	};

	match dbh.insert_entry(&entry) {
		Ok(_) => println!("Entry added to database."),
		Err(e) => println!("Error occurred: {}", e),
	}
}

fn get_boolean_data(name: &str) -> Option<bool> {
	let mut data = String::new();
	loop {
		print!("Did {} occur today [y/n]: ", name);
		io::Write::flush(&mut io::stdout()).expect("flush failed!");
		io::stdin().read_line(&mut data).expect("Invalid input received");
		match data.to_ascii_lowercase().trim() {
			"y" => return Some(true),
			"n" => return Some(false),
			_ => return None,
		}
	}
}

fn get_numeric_data(name: &str) -> Option<f64> {
	let mut data = String::new();
	loop {
		print!("How would you report the value of {}: ", name);
		io::Write::flush(&mut io::stdout()).expect("flush failed!");
		io::stdin().read_line(&mut data).expect("Invalid input received");
		// if f64::from(data)
		if data.trim().len() == 0 {
			return None;
		}
		let num = data.trim().parse::<f64>();
		if num.is_ok() {
			return Some(num.unwrap())
		} else {
			println!("\nData given was not a valid number.")
		}
	}
}

fn analyze_db() {
	println!("Performing analysis...");
	analyze(PathBuf::from("not a real path"));
}

#[cfg(test)]
mod main_tests {
	use super::*;
	use crate::util::test_utils::*;

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