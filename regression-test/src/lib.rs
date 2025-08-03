use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RegType {
    Display,
    Debug,
}

#[derive(Serialize, Deserialize, Debug)]
struct RegEntry {
    #[serde(rename = "type")]
    reg_type: RegType,
    message: String,
}

/// Regression test mode
enum Mode {
    /// We are currently generating the regression test data, and writing it on
    /// disk when appropriate.
    Write,
    /// We are curently comparing previously generated regression test data with
    /// current output, to determine delta.
    Read,
}

pub struct RegTest {
    /// File path to the regression test output
    file_path: PathBuf,
    /// Test mode -- if we are currently generating the regression test data, or
    /// comparing it.
    mode: Mode,
    /// In [Mode::Write]. Caches the entries when generating regression test
    /// data, and written only when this structure goes out of scope or is
    /// manually dropped.
    ///
    /// In [Mode::Read], contains all previously generated regression test data,
    /// and is used to compare with current output.
    buffer: Vec<RegEntry>,
    /// Used in [Mode::Read]. Next regression test to process.
    read_index: usize,
}

impl RegTest {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file_path = path.as_ref().to_path_buf();

        if file_path.exists() {
            // Store all entries in memory
            let file = OpenOptions::new().read(true).open(&file_path)?;

            let mut reader = std::io::BufReader::new(file);

            let buffer = match serde_json::from_reader(&mut reader) {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!(
                        "Failed to read regression test file {}: {}",
                        file_path.display(),
                        e
                    );
                    return Err(e.into());
                }
            };

            Ok(RegTest {
                file_path,
                mode: Mode::Read,
                buffer,
                read_index: 0,
            })
        } else {
            Ok(RegTest {
                file_path,
                mode: Mode::Write,
                buffer: Vec::new(),
                read_index: 0,
            })
        }
    }

    fn regtest_internal(&mut self, message: String, reg_type: RegType) {
        match self.mode {
            Mode::Write => {
                self.buffer.push(RegEntry { reg_type, message });
            }
            Mode::Read => {
                if self.read_index >= self.buffer.len() {
                    panic!("No more regression entries in file, but test expected more.");
                }

                let expected = &self.buffer[self.read_index];
                self.read_index += 1;

                if expected.reg_type != reg_type {
                    panic!(
                        "Regression data generated in different ways: expected {:?}, got {:?}",
                        expected.reg_type, reg_type
                    );
                }

                if expected.message != message {
                    panic!(
                        "Regression message mismatch:\nExpected: {}\nActual:   {}\n\nDiff:\n{}",
                        expected.message,
                        message,
                        diff_lines(&expected.message, &message)
                    );
                }
            }
        }
    }

    pub fn regtest<T: Display>(&mut self, value: T) {
        self.regtest_internal(format!("{}", value), RegType::Display);
    }

    pub fn regtest_dbg<T: Debug>(&mut self, value: T) {
        self.regtest_internal(format!("{:?}", value), RegType::Debug);
    }
}

impl Drop for RegTest {
    fn drop(&mut self) {
        if let Mode::Write = self.mode {
            // Only create/write the file here
            if let Ok(file) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&self.file_path)
            {
                let mut writer = BufWriter::new(file);
                if serde_json::to_writer_pretty(&mut writer, &self.buffer).is_ok() {
                    let _ = writer.flush();
                }
            }
        }
    }
}

fn diff_lines(expected: &str, actual: &str) -> String {
    let exp_lines: Vec<_> = expected.lines().collect();
    let act_lines: Vec<_> = actual.lines().collect();
    let max = exp_lines.len().max(act_lines.len());

    let mut diff = String::new();
    let mut minus_block = Vec::new();
    let mut plus_block = Vec::new();

    for i in 0..max {
        let exp = exp_lines.get(i).unwrap_or(&"");
        let act = act_lines.get(i).unwrap_or(&"");

        if exp != act {
            if !exp.is_empty() {
                minus_block.push(exp);
            }
            if !act.is_empty() {
                plus_block.push(act);
            }
        } else {
            if !minus_block.is_empty() || !plus_block.is_empty() {
                if !minus_block.is_empty() {
                    for line in &minus_block {
                        diff.push_str(&format!("- {}\n", line));
                    }
                    minus_block.clear();
                }
                if !plus_block.is_empty() {
                    for line in &plus_block {
                        diff.push_str(&format!("+ {}\n", line));
                    }
                    plus_block.clear();
                }
            } else {
                diff.push_str(&format!("  {}\n", exp));
            }
        }
    }

    // Flush any remaining blocks
    if !minus_block.is_empty() {
        for line in &minus_block {
            diff.push_str(&format!("- {}\n", line));
        }
    }
    if !plus_block.is_empty() {
        for line in &plus_block {
            diff.push_str(&format!("+ {}\n", line));
        }
    }

    diff
}
