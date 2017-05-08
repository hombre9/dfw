use eval;
use regex::Regex;
use std::collections::HashMap as Map;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(^\$\{?|\$\{)(?P<group_name>\w+)=(?P<pattern>\w+)(\}?$|\})")
                               .unwrap();
    static ref PATTERNS: Map<&'static str, &'static str> = {
        let mut m = Map::new();
        m.insert("ip", r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}");
        m.insert("bridge", r"br-[a-f0-9]{12}");
        m
    };
}

#[derive(Debug)]
pub struct LogLine {
    pub function: String,
    pub regex: bool,
    pub command: String,
    pub eval: Option<String>,
}

impl PartialEq for LogLine {
    fn eq(&self, other: &LogLine) -> bool {
        // If `function` is unequal, we don't have to do further comparisons
        if self.function != other.function {
            return false;
        }

        if self.regex {
            if other.regex {
                // Both are regex, not equal by our definition.
                return false;
            }

            // Handle regex
            let re = Regex::new(&self.command).unwrap();

            // Verify we have a match
            if !re.is_match(&other.command) {
                return false;
            }

            // Check if we have to have constraints to evaluate
            if let Some(ref eval) = self.eval {
                // Get capture groups
                let captures = re.captures(&other.command).unwrap();

                // Try to expand the capture groups used in the eval-string
                let mut expansion = String::new();
                captures.expand(&eval, &mut expansion);

                // Evaluate the string
                let e = eval::eval(&expansion);
                return e.is_ok() && e.unwrap() == eval::to_value(true);
            } else {
                // Nothing to evaluate, `is_match` was successful.
                return true;
            }
        } else {
            if other.regex {
                // We don't want to duplicate the regex handling, just ask `other` for the result.
                return other.eq(self);
            } else {
                // No regex involved, just `command` left to compare
                return self.command == other.command;
            }
        }
    }
}

fn expand_command(command: &str) -> (String, bool) {
    let mut expanded = false;
    (command
         .split(" ")
         .into_iter()
         .map(|e| if !RE.is_match(e) && RE.find(e).is_none() {
                  // Segment of command is not in the form `$group_name=pattern`,
                  // return as is.
                  e.to_owned()
              } else {
                  let c = RE.captures(e).unwrap();

                  // Since the regex matched, both the complete match and the
                  // named groups can't be none, so unwrapping is safe.
                  let c0 = c.get(0).unwrap();
                  let (group_name, pattern) = (c.name("group_name").unwrap().as_str(),
                                               c.name("pattern").unwrap().as_str());

                  // Check if the pattern exists, otherwise leave the segment
                  // unchanged.
                  if let Some(ref pattern) = PATTERNS.get(pattern) {
                      expanded = true;
                      // Match could be in the middle of a string, keep the parts before and after.
                      let (before, after) = (&e[..c0.start()], &e[c0.end()..]);
                      format!(r"{}(?P<{}>{}){}", before, group_name, pattern, after)
                  } else {
                      e.to_owned()
                  }
              })
         .collect::<Vec<_>>()
         .join(" ")
         .to_owned(),
     expanded)
}

pub fn load_log(log_path: &str) -> Vec<LogLine> {

    let file = BufReader::new(File::open(log_path).unwrap());
    let mut v = Vec::new();

    for line in file.lines() {
        if line.is_err() {
            continue;
        }
        let line = line.unwrap();

        // Split line on tabs
        let s = line.split("\t").collect::<Vec<_>>();

        // Line has to be either:
        //     function<TAB>command
        // or
        //     function<TAB>command<TAB>eval
        let eval = match s.len() {
            2 => None,
            3 => Some(s[2].to_owned()),
            _ => panic!("log line split incorrectly"),
        };

        // The command might contain pattern-expansions in the form `$group_name=pattern`.
        let (command, expanded) = expand_command(s[1]);

        let logline = LogLine {
            function: s[0].to_owned(),
            command: command,
            regex: expanded,
            eval: eval,
        };
        v.push(logline);
    }

    v
}