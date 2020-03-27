use std::str::FromStr;

use regex::Regex;

// --------------------------------------------------------------------------- //
/// Une dimension en 2d, c'est un tuple avec deux grandeurs.
// --------------------------------------------------------------------------- //
#[derive(Clone, Copy)]
pub struct Dimension(usize, usize);

static DIM_FMT: &str = r"(?P<WIDTH>\d+),\s*(?P<HEIGHT>\d+)";
lazy_static! {
    static ref DIM_RE : Regex = Regex::new(DIM_FMT).unwrap();
}

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