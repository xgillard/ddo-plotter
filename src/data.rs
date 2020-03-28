use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use regex::Regex;

// --------------------------------------------------------------------------- //
/// A log line outputed by the ddo library solver can have either of the
/// following two formats:
/// *  `Explored 6700, LB 11, UB 12, Fringe sz 90`
/// *  `Final 11, Explored 6790`
// --------------------------------------------------------------------------- //
#[derive(Debug, Clone, Copy)]
pub enum LogLine {
    Ongoing {
        explored: usize,
        lb      : i32,
        ub      : i32,
        fringe  : usize
    },
    Final {
        explored : usize,
        opt_value: i32
    }
}

impl LogLine {
    pub fn explored(&self) -> usize {
        match self {
            LogLine::Ongoing {explored, ..} => *explored,
            LogLine::Final   {explored, ..} => *explored
        }
    }
    pub fn lb(&self) -> i32 {
        match self {
            LogLine::Ongoing {lb, ..}         => *lb,
            LogLine::Final   {opt_value, .. } => *opt_value
        }
    }
    pub fn ub(&self) -> i32 {
        match self {
            LogLine::Ongoing {ub,  ..}        => *ub,
            LogLine::Final   {opt_value, .. } => *opt_value
        }
    }
    pub fn fringe(&self) -> usize {
        match self {
            LogLine::Ongoing {fringe, .. }    => *fringe,
            LogLine::Final   { .. }           => 0
        }
    }
}

// --------------------------------------------------------------------------- //
// Parsing d'une logline
// --------------------------------------------------------------------------- //
static ONGOING_FMT : &str =
    r"Explored (?P<explored>\d+), LB (?P<lb>-?\d+), UB (?P<ub>-?\d+), Fringe sz (?P<fringe>\d+)";
static FINAL_FMT : &str =
    r"Final (?P<opt>-?\d+), Explored (?P<explored>\d+)";

lazy_static! {
    static ref ONGOING_EXP: Regex = Regex::new(ONGOING_FMT).unwrap();
    static ref FINAL_EXP  : Regex= Regex::new(FINAL_FMT).unwrap();
}

impl TryFrom<&str> for LogLine {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(captures) = ONGOING_EXP.captures(value) {
            return Ok(LogLine::Ongoing {
                explored: captures["explored"].parse::<usize>().unwrap(),
                lb      : captures["lb"].parse::<i32>().unwrap(),
                ub      : captures["ub"].parse::<i32>().unwrap(),
                fringe  : captures["fringe"].parse::<usize>().unwrap(),
            });
        }

        if let Some(captures) = FINAL_EXP.captures(value) {
            return Ok(LogLine::Final {
                explored :  captures["explored"].parse::<usize>().unwrap(),
                opt_value: captures["opt"].parse::<i32>().unwrap(),
            });
        }

        Err(())
    }
}

// --------------------------------------------------------------------------- //
/// Une trace, c'est une collection de log lines ...
// --------------------------------------------------------------------------- //
#[derive(Clone, Debug)]
pub struct Trace {
    pub name : Option<String>,
    pub lines: Vec<LogLine>
}

impl Trace {
    pub fn lb_explored(&self) -> Vec<(f64, f64)> {
        self.lines.iter()
            .map(|ll| (ll.explored() as f64, ll.lb() as f64))
            .collect()
    }
    pub fn ub_explored(&self) -> Vec<(f64, f64)> {
        self.lines.iter()
            .map(|ll| (ll.explored() as f64, ll.ub() as f64))
            .collect()
    }
    pub fn fringe_explored(&self) -> Vec<(f64, f64)> {
        self.lines.iter()
            .map(|ll| (ll.explored() as f64, ll.fringe() as f64))
            .collect()
    }
}

// --------------------------------------------------------------------------- //
// Parsing d'une trace
// --------------------------------------------------------------------------- //
impl From<&str> for Trace {
    fn from(lines: &str) -> Self {
        let mut result = Trace{ name: None, lines: vec![]};
        for line in lines.lines() {
            if let Ok(logline) = LogLine::try_from(line) {
                result.lines.push(logline);
            }
        }
        result
    }
}
impl <'a, X: BufRead> From<Lines<X>> for Trace {
    fn from(lines: Lines<X>) -> Self {
        let mut result = Trace{ name: None, lines: vec![]};
        for line in lines {
            let line = line.unwrap();
            if let Ok(logline) = LogLine::try_from(line.as_str()) {
                result.lines.push(logline);
            }
        }
        result
    }
}
impl From<File> for Trace {
    fn from(file: File) -> Self {
        BufReader::new(file).lines().into()
    }
}
impl TryFrom<&Path> for Trace {
    type Error=std::io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let stem  = path.file_stem().map(|f| f.to_string_lossy().to_string());
        let mut trace = Trace::from(File::open(path)?);
        trace.name = stem;
        Ok(trace)
    }
}


#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use crate::data::{LogLine, Trace};

    #[test]
    fn parse_final_line() {
        let parsed = LogLine::try_from("Final 11, Explored 6790").unwrap();

        assert_eq!(11,   parsed.lb());
        assert_eq!(11,   parsed.ub());
        //assert_eq!(0 ,   parsed.fringe());
        assert_eq!(6790, parsed.explored());
    }
    #[test]
    fn parse_ongoing_line() {
        let line   = "Explored 6700, LB 11, UB 12, Fringe sz 90";
        let parsed = LogLine::try_from(line).unwrap();

        assert_eq!(11,   parsed.lb());
        assert_eq!(12,   parsed.ub());
        //assert_eq!(90,   parsed.fringe());
        assert_eq!(6700, parsed.explored());
    }
    #[test]
    fn parse_final_line_with_negatives() {
        let line   = "Final -11, Explored 6790";
        let parsed = LogLine::try_from(line).unwrap();

        assert_eq!(-11,  parsed.lb());
        assert_eq!(-11,  parsed.ub());
        //assert_eq!(0 ,   parsed.fringe());
        assert_eq!(6790, parsed.explored());
    }
    #[test]
    fn parse_ongoing_line_with_negatives() {
        let line   = "Explored 6700, LB -11, UB -12, Fringe sz 90";
        let parsed = LogLine::try_from(line).unwrap();

        assert_eq!(-11,  parsed.lb());
        assert_eq!(-12,  parsed.ub());
        //assert_eq!(90,   parsed.fringe());
        assert_eq!(6700, parsed.explored());
    }

    #[test]
    fn when_it_fails() {
        let line   = "Coucou ca va ?";
        let parsed = LogLine::try_from(line);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_empty_trace() {
        let log   = "";
        let trace = Trace::from(log);

        assert_eq!(0, trace.lines.len())
    }

    #[test]
    fn parse_trace() {
        let log   = "
Explored 5900, LB 11, UB 14, Fringe sz 890
Explored 6000, LB 11, UB 14, Fringe sz 790
Explored 6100, LB 11, UB 14, Fringe sz 690
Explored 6200, LB 11, UB 14, Fringe sz 590
Explored 6300, LB 11, UB 14, Fringe sz 490
Explored 6400, LB 11, UB 13, Fringe sz 390
Explored 6500, LB 11, UB 13, Fringe sz 290
Explored 6600, LB 11, UB 12, Fringe sz 190
Explored 6700, LB 11, UB 12, Fringe sz 90
Final 11, Explored 6790
Optimum 11 computed in 5.042205s with 1 threads
### Solution: ################################################
 4 13 27 31 45 56 78 88 102 124 133
";
        let trace = Trace::from(log);

        assert_eq!(10, trace.lines.len());
    }

}