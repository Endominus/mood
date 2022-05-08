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
