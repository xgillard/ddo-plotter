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

#[macro_use]
extern crate lazy_static;
extern crate structopt;
extern crate regex;

use std::io::{stdin, BufReader, BufRead, BufWriter};
use std::fs::File;
use structopt::StructOpt;
use crate::trace::{Trace, Dimension};

mod trace;

/// Parse a DDO trace and process it to produce graphs.
#[derive(StructOpt)]
struct Args {
    /// If set, the path to a file containg the text of a ddo trace
    #[structopt(name="input", short, long)]
    input_fname: Option<String>,
    /// If set, the graph will be saved in svg at the specified location.
    #[structopt(name="svg", short, long)]
    graph_fname: Option<String>,
    /// If set, the complete trace data will be save to json for further processing
    #[structopt(name="json", short, long)]
    json_fname : Option<String>,
    /// If set, the dimension of the terminal (otherwise it will attempt to auto detect)
    #[structopt(name="dimension", short, long)]
    dimension  : Option<Dimension>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::from_args();

    let trace =
        if let Some(fname) = args.input_fname {
            Trace::from(BufReader::new(File::open(fname)?).lines())
        } else {
            Trace::from(BufReader::new(stdin()).lines())
        };

    if let Some(fname) = args.graph_fname {
        trace.plot_to_file(fname.as_str());
    }

    if let Some(fname) = args.json_fname {
        let out = BufWriter::new(File::create(fname)?);
        serde_json::to_writer(out, &trace).expect("Could not write JSON")
    }

    trace.plot_to_term(args.dimension);
    Ok(())
}
