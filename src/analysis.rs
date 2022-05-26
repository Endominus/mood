use crate::prelude::*;

const ROLLING_AVERAGE_WINDOW: u32 = 14;
const ROLLING_AVERAGE_MINIMUM: usize = 10;
const ROLLING_AVERAGE_DISCARD: usize = 4;

pub fn analyze(path: PathBuf) {
	let dbh = DbHandler::new(path);
	// let entries = dbh.get_entries().unwrap();
	let fields = dbh.get_fields().unwrap(); // only output fields?
	// let vlobf = calculate_lobf_from_entries(&entries);

	// let start_date = entries.first().unwrap().date.clone();
	// let end_date = entries.last().unwrap().date.clone();
	let mut all_trends = Vec::new();

	for field in fields {
		let trends = find_all_trends(&dbh, &field.name);
		all_trends.append(&mut trends.unwrap());
		// baseline.insert(field.name.clone(), trend);
	}

}

fn calculate_std_dev(entries: &Vec<(u32, f64)>, slope: f64, intercept: f64) -> f64 {
	let mut total = 0.0;
	let n = entries.len() as f64;
	let start_date = entries.first().unwrap().0;
	for (date, value) in entries {
		let time_delta = (date - start_date) as f64;
		total += (value - (intercept + slope * time_delta)).powi(2)/n;
	}

	total.sqrt()
}

fn calculate_single_lobf(entries: &Vec<(u32, f64)>) -> (f64, f64) {
	let mut ws = WorkSet::default();
	let start_date = entries.first().unwrap().0;

	for (date, value) in entries {
		ws.dates.push(date - start_date);
		ws.values.push(*value);
	}

	let sum_date = ws.dates.iter().sum::<u32>();
	ws.date_prime = (sum_date as f64) / (ws.dates.len() as f64);
	let sum_values: f64 = ws.values.iter().sum();
	ws.value_prime = sum_values / (ws.dates.len() as f64);

	let mut total_date_deviance = 0.0;
	let mut total_value_deviance = 0.0;
	for (index, date) in ws.dates.iter().enumerate() {
		let date_deviance = (f64::from(*date) - ws.date_prime).powi(2);
		total_date_deviance += date_deviance;
		let value_deviance = (f64::from(*date) - ws.date_prime) * (ws.values.get(index).unwrap() - ws.date_prime);
		total_value_deviance += value_deviance;
	}
	
	let slope = total_value_deviance / total_date_deviance;
	let intercept = ws.value_prime - slope * ws.date_prime;
	(slope, intercept)
}

fn find_all_trends(dbh: &DbHandler, field: &String) -> Result<Vec<Trend>> {
	let (start, end) = dbh.get_range();
	let mut date_vals = dbh.get_numeric_values_between_dates(&field, start, end).unwrap();
	if date_vals.len() < 15 {
		return Err(anyhow!("Too little data to find trend for {}", field))
	}
	let mut trends = Vec::new();
	// let mut old_trend = Trend::default();
	while date_vals.len() > 14 {
		let mut new_trend = find_trends(dbh, field, date_vals.first().unwrap().0, date_vals.get(13).unwrap().0).unwrap();
		new_trend = extend_trend(&new_trend, field, dbh);

		date_vals = dbh.get_numeric_values_between_dates(&field, new_trend.end, end).unwrap();
		if validate_trend(&new_trend) {
			trends.push(new_trend);
		}
	}

	Ok(trends)
}

fn validate_trend(trend: &Trend) -> bool {
	let sufficient_length = trend.end - trend.start >= 30;
	let significant_slope = trend.slope.abs() > 0.1;
	let narrow_deviation = true; //How do we define this?

	sufficient_length && significant_slope && narrow_deviation
}

fn extend_trend(trend: &Trend, field: &String, dbh: &DbHandler) -> Trend {
	let (_, end) = dbh.get_range();
	let date_vals = dbh.get_numeric_values_between_dates(&field, trend.end, end).unwrap();
	let mut trend = trend.clone();
	// let mut accumulated_error = 0.0;
	let mut total_outliers = 0.0;
	let mut i = 0;
	let mut correction = false;
	let mut range = dbh.count_entries(field, trend.start, trend.end).unwrap() as f64;
	let mut confidence = range / trend.stddev;
	while i < date_vals.len() {
		// let dv = date_vals.get(i);
		// let date = dv.unwrap().0;
		// let val = dv.unwrap().1;
		let (date, val) = date_vals.get(i).unwrap();
		let time_delta = (date - trend.start) as f64;
		let expected_val = trend.orig_val + trend.slope * time_delta;
		// accumulated_error += val - expected_val;
		if (val - expected_val).abs() > 2.0 * trend.stddev {
			total_outliers += 1.0
		}
		// if accumulated_error.abs() > expected_val && i > 10 {
		if i > 6 && total_outliers / (i as f64 + range) > 0.05 {
			let new_trend = find_trends(dbh, field, trend.start, *date).unwrap();
			let new_range = dbh.count_entries(field, trend.start, new_trend.end).unwrap() as f64;
			let new_confidence = new_range / new_trend.stddev;
			let length_factor = new_range / (range * 1.1);
			if new_confidence >= confidence * length_factor {
				range = new_range;
				confidence = new_confidence;
				trend = new_trend;
				total_outliers = 0.0;
			} else {
				correction = true;
				break
			}
		}

		i += 1;
	}

	if correction {
		// let pos = accumulated_error > 0.0;
		while i > 0 {
			let (date, val) = date_vals.get(i).unwrap();
			let time_delta = (date - trend.start) as f64;
			let expected_val = trend.orig_val + trend.slope * time_delta;
			// accumulated_error -= val - expected_val;
			
			if (val - expected_val).abs() > 2.0 * trend.stddev {
				total_outliers -= 1.0
			}
			// if (pos && accumulated_error < 0.0)
			// || (!pos && accumulated_error > 0.0) {
			if total_outliers / (i as f64 + 14.0) < 0.05 {
				break
			}
	
			i -= 1;
		}
	}

	i = i.clamp(0, date_vals.len() - 1);
	trend.end = date_vals.get(i).unwrap().0;

	trend
}

fn detect_outliers(dbh: &DbHandler, field: &String) -> Vec<u32> {
	let mut spikes = Vec::new();
	let entries = dbh.get_numeric_values(field).unwrap();

	let mut active: Vec<(u32, f64)> = Vec::new();
	let mut recheck = Vec::new();

	for (date, val) in entries.iter() {
		while !active.is_empty() &&
		active.first().unwrap().0 < date - ROLLING_AVERAGE_WINDOW {
			active.remove(0);
		}

		if active.len() < ROLLING_AVERAGE_DISCARD {
			active.push((*date, *val));
			recheck.clear();
			continue;
		}
		if active.len() < ROLLING_AVERAGE_MINIMUM {
			active.push((*date, *val));
			recheck.push((*date, *val));
			continue;
		}

		let (slope, intercept) = calculate_single_lobf(&active);
		let stddev = calculate_std_dev(&active, slope, intercept);
		active.push((*date, *val));
		let time_delta = date - active.first().unwrap().0;
		let expected = intercept + slope * time_delta as f64;

		if (*val - expected).abs() > 2.5 * stddev {
			spikes.push(*date);
		}

		while !recheck.is_empty() {
			let point = recheck.pop().unwrap();
			let time_delta = point.0 - active.first().unwrap().0;
			let expected = intercept + slope * time_delta as f64;
			if (point.1 - expected).abs() > 2.0 * stddev {
				spikes.push(point.0);
			}
		}
	}

	spikes
}

fn _remove_outliers(range: &Vec<(i32, f64)>) -> Vec<(i32, f64)> {
	let mut v = range.clone();
    let max = range.iter().enumerate().max_by(|orig, new| orig.1.1.partial_cmp(&new.1.1).unwrap() ).unwrap().0;
	v.remove(max);
    let min = range.iter().enumerate().min_by(|orig, new| orig.1.1.partial_cmp(&new.1.1).unwrap() ).unwrap().0;
	v.remove(min);
	v
}

fn find_trends(dbh: &DbHandler, field: &String, start_date: u32, end_date: u32) -> Result<Trend> {
	let date_vals = dbh.get_numeric_values_between_dates(&field, start_date, end_date).unwrap();
	// let date_vals = remove_outliers(&date_vals);
	// if date_vals.len() < 15 {
	// 	return Err(anyhow!("Too little data to find trend for {}", field))
	// }
	let (lobf, intercept) = calculate_single_lobf(&date_vals);
	let new_trend = Trend {
		start: start_date,
		end: end_date,
		orig_val: intercept,
		slope: lobf,
		stddev: calculate_std_dev(&date_vals, lobf, intercept),
	};
	
	Ok(new_trend)
}


#[derive(Default)]
struct WorkSet {
	dates: Vec<u32>,
	values: Vec<f64>,
	date_prime: f64,
	value_prime: f64,
	// date_deviance: f64,
	// value_deviance: f64,
}

#[cfg(test)]
mod analysis_test {
	use super::*;
	use crate::test_utils::*;

	#[test]
	fn test_slope_derivation() {
		let path = PathBuf::from("test_trend.db");
		let dbh = if !path.exists() {
			let dbh = setup_db("test_trend.db");
			import_csv(&dbh, "test_trends.csv");
			dbh
		} else {
			DbHandler::new(path)
		};

		let trend = find_trends(&dbh, &String::from("switch"), 22120, 22169).unwrap();
		// println!("{:#?}", trend);

		assert!(trend.slope > 0.4);
		assert!(trend.slope < 0.6);
		assert!(trend.stddev < 6.0);

		let trend = find_trends(&dbh, &String::from("switch"), 22120, 22134).unwrap();
		let trend = extend_trend(&trend, &String::from("switch"), &dbh);
		// println!("{:#?}", trend);
		assert!(trend.slope > 0.4);
		assert!(trend.slope < 0.6);
		assert!(trend.stddev < 6.0);
	}

	#[test]
	fn test_find_simple_trends() {
		let path = PathBuf::from("test_trend.db");
		// let mut instant = time::Instant::now();
		let dbh = if !path.exists() {
			let dbh = setup_db("test_trend.db");
			// println!("Creating database took {}", time::Instant::now() - instant);
			// instant = time::Instant::now();
			import_csv(&dbh, "test_trends.csv");
			// println!("Importing csv took {}", time::Instant::now() - instant);
			dbh
		} else {
			DbHandler::new(path)
		};
		// let start_date = Date::from_julian_day(22120).unwrap();
		// let end_date = Date::from_julian_day(22298).unwrap();

		let cols = vec![("easy", 0.5), ("med", 0.3), ("hard", 0.3)];

		for (col, slope) in cols {
			// instant = time::Instant::now();
			let trends = find_all_trends(&dbh, &String::from(col));
			// println!("Finding trends took {}", time::Instant::now() - instant);
			assert!(trends.is_ok());
			let trends = trends.unwrap();
			// for trend in &trends {
			// 	println!("Slope/stddev of {}:\t{:.4}\t{:.4}\t{}", col, &trend.slope, &trend.stddev, &trend.end - &trend.start);
			// }
			// assert!(trends.len() == 1);
			let trend = trends.first().unwrap();
			assert!(trend.slope > slope - 0.1);
			assert!(trend.slope < slope + 0.1);
		}
		let trends = find_all_trends(&dbh, &String::from("switch"));
		assert!(trends.is_ok());
		let trends = trends.unwrap();
		assert!(trends.len() == 2);
		let trend = trends.first().unwrap();
		// println!("Slope/stddev of {}:\t{:.4}\t{:.4}\t{}", "switch", &trend.slope, &trend.stddev, &trend.start);
		assert!(trend.slope > 0.4);
		assert!(trend.slope < 0.6);
		let trend = trends.last().unwrap();
		// println!("Slope/stddev of {}:\t{:.4}\t{:.4}\t{}", "switch", &trend.slope, &trend.stddev, &trend.start);
		assert!(trend.slope > -0.3);
		assert!(trend.slope < -0.1);
	}

	#[test]
	fn test_find_outliers() {
		let path = PathBuf::from("test_trend.db");
		let dbh = if !path.exists() {
			let dbh = setup_db("test_trend.db");
			import_csv(&dbh, "test_trends.csv");
			dbh
		} else {
			DbHandler::new(path)
		};

		// let outliers = detect_outliers(&dbh, &String::from("easy"));
		// println!("For easy: {:#?}", outliers.len());
		// let outliers = detect_outliers(&dbh, &String::from("med"));
		// println!("For medi: {:#?}", outliers.len());
		// let outliers = detect_outliers(&dbh, &String::from("hard"));
		// println!("For hard: {:#?}", outliers.len());
		let outliers = detect_outliers(&dbh, &String::from("outliers"));
		assert!(outliers.contains(&22129));
		assert!(outliers.contains(&22138));
		// println!("{:#?}", outliers.len());
		// assert!(outliers.len() == 2);

	}
}