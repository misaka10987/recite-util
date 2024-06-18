use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{BufReader, BufWriter},
};

use inquire::{Confirm, Text};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
struct Meta {
    pub cnt: i16,
    pub aka: HashSet<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Entry {
    pub en: String,
    pub zh: String,
    meta_str: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub meta: Meta,
}

impl Entry {
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        self.meta = if let Some(s) = &self.meta_str {
            serde_json::from_str(s)?
        } else {
            Meta::default()
        };
        Ok(())
    }
    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.meta_str = Some(serde_json::to_string(&self.meta)?);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let arg = env::args().last().unwrap_or("./word.csv".to_owned());
    let file = fs::File::open(&arg)?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));
    let mut v: Vec<Entry> = vec![];
    for row in reader.deserialize() {
        v.push(row?)
    }
    for row in &mut v {
        row.load()?;
    }
    for entry in &mut v {
        if entry.meta.cnt > 3 {
            continue;
        }
        let ans = Text::new(&format!("zh = {}, en = ?", entry.zh)).prompt()?;
        if ans == entry.en {
            println!("correct: en = {}", entry.en);
            entry.meta.cnt += 1;
            continue;
        }
        println!("{ans} does not match the entry: {}", entry.en);
        let add = Confirm::new(&format!("would you like to add {ans} as a correct answer?"))
            .with_default(false)
            .prompt()?;
        if add {
            entry.meta.cnt += 1;
            entry.meta.aka.insert(ans);
        }
    }
    for row in &mut v {
        row.save()?;
    }
    let file = OpenOptions::new().write(true).append(false).open(&arg)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));
    for entry in v {
        writer.serialize(entry)?;
    }
    Ok(())
}
