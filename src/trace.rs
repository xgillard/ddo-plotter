// Copyright 2020 Xavier Gillard
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::convert::TryFrom;
use regex::Regex;
use serde::{Serialize, Deserialize};
use plotlib::scatter::{Scatter, Style};
use plotlib::style::{Point, Marker};
use plotlib::page::Page;
use plotlib::view::ContinuousView;
use std::cmp::{min, max};
use std::io::{BufRead, Lines};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trace {
    lines: Vec<LogLine>
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

    pub fn plot_to_term(&self, fringe: bool, dim: Option<Dimension>) {
        if fringe {
            FringePlotRepr::from(self).plot_to_term(dim);
        } else {
            TracePlotRepr::from(self).plot_to_term(dim);
        }
    }

    pub fn plot_to_file(&self, fringe: bool, fname: &str) {
        if fringe {
            FringePlotRepr::from(self).plot_to_file(fname);
        } else {
            TracePlotRepr::from(self).plot_to_file(fname);
        }
    }
}

impl From<&str> for Trace {
    fn from(lines: &str) -> Self {
        let mut result = Trace{ lines: vec![]};
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
        let mut result = Trace{ lines: vec![]};
        for line in lines {
            let line = line.unwrap();
            if let Ok(logline) = LogLine::try_from(line.as_str()) {
                result.lines.push(logline);
            }
        }
        result
    }
}

static DIM_FMT: &str = r"(?P<WIDTH>\d+),\s*(?P<HEIGHT>\d+)";
lazy_static! {
    static ref DIM_RE : Regex = Regex::new(DIM_FMT).unwrap();
}

#[derive(Clone, Copy)]
pub struct Dimension(usize, usize);

impl FromStr for Dimension {
    type Err = &'static str;
    fn from_str(txt: &str) -> Result<Dimension, Self::Err> {
        if let Some(caps) = DIM_RE.captures(txt) {
            let w = caps["WIDTH"].parse::<usize>().unwrap();
            let h = caps["HEIGHT"].parse::<usize>().unwrap();
            Ok(Dimension(w, h))
        } else {
            Err("Input does not conform to format 'width,height'")
        }
    }
}

struct FringePlotRepr{
    y_range     : (f64, f64),
    fsz_explored: Vec<(f64, f64)>
}
impl FringePlotRepr {
    fn set_fsz_size(lb: Scatter) -> Scatter {
        lb.style(Style::new().marker(Marker::Square).colour("#C1EBE1"))
    }
    pub fn plot_to_term(&self, dim: Option<Dimension>) {
        let (w, h) =
            if let Some(Dimension(ww, hh)) = dim {
                (ww, hh)
            } else if let Some((ww, hh)) = term_size::dimensions() {
                (ww - 10, hh - 10)
            } else {
                (45, 15)
            };

        let w      = w as u32;
        let h      = h as u32;
        let (x ,y) = self.y_range;
        let fsz    = Self::set_fsz_size(Scatter::from_slice(&self.fsz_explored));
        let view   = ContinuousView::new()
            .add(&fsz)
            .y_range(x, y)
            .x_label("Explored Nodes")
            .y_label("Fringe Size");

        println!("{}", Page::single(&view).dimensions(w, h).to_text().unwrap());
    }

    pub fn plot_to_file(&self, fname: &str) {
        let (x ,y) = self.y_range;
        let fsz    = Self::set_fsz_size(Scatter::from_slice(&self.fsz_explored));
        let view   = ContinuousView::new()
            .add(&fsz)
            .y_range(x, y)
            .x_label("Explored Nodes")
            .y_label("Fringe Size");

        Page::single(&view).save(fname).expect("Could not save");
    }
}
impl From<&Trace> for FringePlotRepr {
    fn from(trace: &Trace) -> Self {
        let low   = trace.lines.iter().map(|ll| ll.fringe()).min().unwrap_or(0);
        let high  = trace.lines.iter().map(|ll| ll.fringe()).max().unwrap_or(0);
        let range = (low as f64 - 1.0, high as f64 + 1.0);

        FringePlotRepr {
            y_range     : range,
            fsz_explored: trace.fringe_explored()
        }
    }
}

struct TracePlotRepr {
    y_range    : (f64, f64),
    ub_explored: Vec<(f64, f64)>,
    lb_explored: Vec<(f64, f64)>
}
impl TracePlotRepr {
    fn set_lb_style(lb: Scatter) -> Scatter {
        lb.style(Style::new().marker(Marker::Circle).colour("#C1EBE1"))
    }
    fn set_ub_style(ub: Scatter) -> Scatter {
        ub.style(Style::new().marker(Marker::Cross).colour("#90B9A9"))
    }
    pub fn plot_to_term(&self, dim: Option<Dimension>) {
        let (w, h) =
            if let Some(Dimension(ww, hh)) = dim {
                (ww, hh)
            } else if let Some((ww, hh)) = term_size::dimensions() {
                (ww - 10, hh - 10)
            } else {
                (45, 15)
            };

        let w      = w as u32;
        let h      = h as u32;
        let (x ,y) = self.y_range;
        let lbs    = Self::set_lb_style(Scatter::from_slice(&self.lb_explored));
        let ubs    = Self::set_ub_style(Scatter::from_slice(&self.ub_explored));
        let view   = ContinuousView::new()
            .add(&lbs)
            .add(&ubs)
            .y_range(x, y)
            .x_label("Explored Nodes")
            .y_label("Bound Value");

        println!("{}", Page::single(&view).dimensions(w, h).to_text().unwrap());
    }
    pub fn plot_to_file(&self, fname: &str) {
        let (x ,y) = self.y_range;
        let lbs    = Self::set_lb_style(Scatter::from_slice(&self.lb_explored));
        let ubs    = Self::set_ub_style(Scatter::from_slice(&self.ub_explored));
        let view   = ContinuousView::new()
            .add(&lbs)
            .add(&ubs)
            .y_range(x, y)
            .x_label("Explored Nodes")
            .y_label("Bound Value");

        Page::single(&view).save(fname).expect("Could not save");
    }
}
impl From<&Trace> for TracePlotRepr {
    fn from(trace: &Trace) -> Self {
        let lowest_lb = trace.lines.iter().map(|ll| ll.lb()).min().unwrap_or(0);
        let lowest_ub = trace.lines.iter().map(|ll| ll.ub()).min().unwrap_or(0);
        let highest_lb= trace.lines.iter().map(|ll| ll.lb()).max().unwrap_or(0);
        let highest_ub= trace.lines.iter().map(|ll| ll.ub()).max().unwrap_or(0);

        let range = (min(lowest_ub, lowest_lb) as f64 - 1.0, max(highest_ub, highest_lb) as f64 + 1.0);

        TracePlotRepr {
            y_range    : range,
            lb_explored: trace.lb_explored(),
            ub_explored: trace.ub_explored()
        }
    }
}

/// A log line outputed by the ddo library solver can have either of the
/// following two formats:
/// *  `Explored 6700, LB 11, UB 12, Fringe sz 90`
/// *  `Final 11, Explored 6790`
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[cfg(test)]
mod test {
    use std::convert::{TryFrom};
    use crate::trace::{LogLine, Trace};

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