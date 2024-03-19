use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::str;
use log::warn;

pub struct Plugin {
    /// Path to the executable for this plugin
    pub path: OsString,
    /// Elements this plugin handles 
    handles: Vec<String>,
}

impl Plugin {
    /// Get plugins from path.
    /// Path can be:
    /// 1. absolute/relative path to a directory
    /// 2. absolute/relative path to an executable file
    /// 3. executable command (found in $PATH)
    pub fn init(path: &str) -> Result<Vec<Plugin>, io::Error> {
        let mut result: Vec<Plugin> = Vec::new();
        let mut commands: Vec<OsString> = Vec::new();
        let path_absolute = fs::canonicalize(path);
        if path_absolute.is_ok(){
            // option 1 or 2 
            let path_absolute = path_absolute.unwrap();
            if path_absolute.is_dir() {
                for entry in fs::read_dir(path_absolute)? {
                    let entry = entry?;
                    let p = fs::canonicalize(entry.path())?;
                    if p.is_file() {
                        commands.push(p.as_os_str().to_os_string());
                    }
                }
            } else {
                commands.push(OsString::from(path_absolute));
            }
        } else { 
            // option 3
            commands.push(OsString::from(path));
        }

        for i in 0..commands.len() {
            let entry = &commands[i];
            let output = Command::new(entry).arg("elements").output();
            if output.is_err() {
                warn!(
                    "Error reading output from: {} -- {}",
                    entry.to_str().unwrap(),
                    output.unwrap_err());
                continue;
            }
            let output = output.unwrap().stdout;
            let output: Vec<&str> = str::from_utf8(&output)
                .unwrap().trim().split(" ").collect();
            let mut handles: Vec<String> = Vec::new();
            for h in output {
                handles.push(h.to_string());
            }
            if handles.len() > 0 {
                let plug = Plugin {
                    path: entry.to_os_string(), 
                    handles,
                };
                result.push(plug);
            }
        }
        Ok(result)
    }

    pub fn handles(&self, name: &str) -> bool {
        for elem in &self.handles {
            if elem.trim() == name.trim() {
                return true;
            }
        }
        return false;
    }

    pub fn execute(
        &self, elem_name: &str, elem_attr: &str, elem_cont: &str) 
        -> Result<String, Box<dyn Error>> {
        let seperator = "\r\n\r\n";

        let mut child = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(elem_name.as_bytes())
            .expect("Failed to write to stdin");
        stdin.write_all(seperator.as_bytes())
            .expect("Failed to write to stdin");
        stdin.write_all(elem_attr.as_bytes())
            .expect("Failed to write to stdin");
        stdin.write_all(seperator.as_bytes())
            .expect("Failed to write to stdin");
        stdin.write_all(elem_cont.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush()?;
        // stdin is closed when dropped
        drop(stdin); 


        let output = child.wait_with_output()?;
        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    }
}

pub struct Config {
    /// Source,
    pub src: String,
    /// Destination
    pub dst: String,
    /// If true use long form for empty elements.
    pub long_empty: bool,
    /// If true disable enable special elements.
    pub disable_special_elements: bool,
    /// Plugins 
    pub plugins: Vec<Plugin>,
}

struct Element {
    start_idx: usize,
    start_body_idx: usize,
    end_idx: usize,
}

/// returns index of the last byte of the converted element 
fn handle_element(
    doc: &mut String, element: &Element, config: &Config) -> usize {
    let mut name = "";
    let mut attributes = "";

    if element.start_idx + 1 < element.start_body_idx {
        let mut name_end = element.start_idx+2;
        while name_end < element.start_body_idx {
            if doc.as_bytes()[name_end].is_ascii_whitespace() {
                break;
            } 
            name_end += 1;
        }
        name = &doc[element.start_idx+1..name_end];
        if name_end+1 < element.start_body_idx {
            attributes = &doc[name_end+1..element.start_body_idx];
        }
    }


    if !config.disable_special_elements {
        if name == "$o" {
            doc.replace_range(element.start_idx..=element.end_idx, "{");
            return element.start_idx;
        } else if name == "$c" {
            doc.replace_range(element.start_idx..=element.end_idx, "}");
            return element.start_idx;
        } else if name == "$s" {
            doc.replace_range(element.start_idx..=element.end_idx, "\\");
            return element.start_idx;
        } else if name == "$" {
            doc.replace_range(element.start_idx..=element.end_idx, "");
            return element.start_idx;
        }
    }

    let content = if element.start_body_idx + 1 < element.end_idx {
        &doc[(element.start_body_idx+1)..element.end_idx]
    } else {""};

    for plugin in &config.plugins {
        if plugin.handles(name){
            let result = plugin.execute(name, attributes, content)
                .expect("Failed to execute plugin");
            let len = result.len();
            doc.replace_range(element.start_idx..=element.end_idx, &result);
            return if len == 0 {element.start_idx} else {
                element.start_idx + len - 1
            }

        }
    }

    if content.is_empty() && !config.long_empty {
        doc.replace_range(element.start_idx..element.start_idx+1, "<");
        doc.replace_range(
            element.start_body_idx..element.start_body_idx+1, "/");
        doc.replace_range(element.end_idx..element.end_idx+1, ">");
        return element.end_idx;
    }
    let name_len = name.len();
    let end = format!("</{}>", name);
    doc.replace_range(element.start_idx..element.start_idx+1, "<");
    doc.replace_range(
            element.start_body_idx..element.start_body_idx+1, ">");
    doc.replace_range(element.end_idx..element.end_idx+1, &end);
    return element.end_idx + 3 + name_len - 1;
}

pub fn convert(doc: &mut String, config: &Config) 
    -> Result<(), Box<dyn Error>> {
    let mut stack: Vec<Element> = Vec::new();
    let mut i = 0usize;
    let mut c: u8;
    // push document node
    while i < doc.as_bytes().len() {
        c = doc.as_bytes()[i];
        if c == b'\\' { 
            if i + 1 < doc.as_bytes().len() && doc.as_bytes()[i+1] == b'%' {
               // remove \%
               doc.replace_range(i..i+2, "");
               // find next \%
               let token_idx = &doc[i..].find("\\%");
               if let Some(n) = token_idx {
                   i = i + n;
                   doc.replace_range(i..i+2, "");
               } else {
                   warn!("single \\%");
               }
              continue;
            } else {
                stack.push(Element {
                    start_idx: i,
                    start_body_idx: 0,
                    end_idx: 0,
                });
            }
        } else if c == b'{' {
            let last_elem = match stack.last_mut() {
                Some(v) => v,
                None => {
                    let line = &doc[i..].lines().next().unwrap();
                    warn!("{{ that does not start a body: {}", line);
                    i += 1;
                    continue
                },
            };
            if last_elem.start_body_idx != 0 {
                let line = &doc[i..].lines().next().unwrap();
                warn!("{{ that does not start a body: {}", line);
            } else {
                last_elem.start_body_idx = i;
            }
        } else if c == b'}' {
            let last_elem = match stack.last_mut() {
                Some(v) => v,
                None => {
                    let line = &doc[i..].lines().next().unwrap();
                    warn!("Found }} but no element is left: {}", line);
                    i += 1;
                    continue
                },
            };
            if last_elem.start_body_idx == 0 {
                let line = &doc[i..].lines().next().unwrap();
                warn!("Found }} but no element is left: {}", line);
            } else {
                last_elem.end_idx = i;
                i = handle_element(doc, last_elem, config);
                stack.pop();
            }
        }
        i += 1;
    }
    Ok(())
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    // read input
    let mut doc = if config.src == "-" {
        io::read_to_string(io::stdin())?
    } else {
        fs::read_to_string(&config.src)?
    };

    convert(&mut doc, config)?;

    // write output
    if config.dst == "-" {
        io::stdout().write_all(doc.as_bytes())?;
    } else {
        fs::write(&config.dst, doc)?;
    }
    Ok(())
}
