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
extern crate regex;
extern crate structopt;

use std::convert::TryFrom;
use std::path::Path;

use plotlib::page::Page;
use structopt::StructOpt;

use crate::data::Trace;
use crate::repr::{bounds_view, fringe_view};

mod config;
mod data;
mod repr;

/// Parse a DDO trace and process it to produce graphs.
#[derive(StructOpt)]
struct Args {
    /// If set, the path to a file containg the text of a ddo trace
    #[structopt(name="input", short, long)]
    input: Vec<String>,
    /// If set, the graph will be saved in svg at the specified location.
    #[structopt(name="output", short, long)]
    output: String,
    /// If set, prints the evolution of the fringe size
    #[structopt(name="fringe", short, long)]
    fringe     : bool,
}

fn main() {
    let args = Args::from_args();

    let traces = args.input.iter().map(|fname|
            Trace::try_from(Path::new(fname)).expect("Cannot open file")
    ).collect::<Vec<Trace>>();

    let view =
        if args.fringe {
            fringe_view(&traces)
        } else {
            bounds_view(&traces)
        };

    Page::single(&view).save(&args.output).expect("Cannot save output");
}
