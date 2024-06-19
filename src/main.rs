use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::{self, OpenOptions},
    io::{BufReader, BufWriter},
};

use clearscreen::clear;
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
enum Mode {
    E2Z,
    Z2E,
}

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
    pub fn prompt(&mut self, mode: Mode) -> Result<(), Box<dyn Error>> {
        if self.meta.cnt >= 3 {
            return Ok(());
        }
        let (q, a) = match mode {
            Mode::E2Z => (&self.en, &self.zh),
            Mode::Z2E => (&self.zh, &self.en),
        };
        println!();
        let res = Select::new(&format!("{q}?"), vec!["check", "pass"]).prompt()?;
        if res == "pass" {
            println!("NB: {q} -> {a}");
            return Ok(());
        }
        let show = if self.meta.aka.is_empty() {
            format!("ans = {a}")
        } else {
            format!("ans = {a}, aka = [{:?}]", self.meta.aka)
        };
        let res = Select::new(&show, vec!["right", "wrong", "add aka"]).prompt()?;
        match res {
            "right" => self.meta.cnt += 1,
            "wrong" => (),
            "add aka" => {
                let aka = Text::new(&format!("add alternative for {q} -> {a}")).prompt()?;
                self.meta.aka.insert(aka);
            }
            _ => (),
        };
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let env = env::args();
    let res = Select::new("option", vec!["en2zh", "zh2en", "quit"]).prompt()?;
    let mode = match res {
        "en2zh" => Mode::E2Z,
        "zh2en" => Mode::Z2E,
        _ => return Ok(()),
    };
    let arg = if env.len() > 1 {
        env.last().unwrap()
    } else {
        "./word.csv".to_owned()
    };
    let file = fs::File::open(&arg)?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));
    let mut v: Vec<Entry> = vec![];
    for row in reader.deserialize() {
        v.push(row?)
    }
    for row in &mut v {
        row.load()?;
    }
    for row in &mut v {
        row.prompt(mode)?;
    }
    for row in &mut v {
        row.save()?;
    }
    if !Confirm::new(&format!(
        "completed reciting {arg}, write statistics to file?"
    ))
    .with_default(true)
    .prompt()?
    {
        return Ok(());
    }
    println!("saving to file, please do not exit...");
    let file = OpenOptions::new().write(true).append(false).open(&arg)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));
    for entry in v {
        writer.serialize(entry)?;
    }
    clear()?;
    Ok(())
}
