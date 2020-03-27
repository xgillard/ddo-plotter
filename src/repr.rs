use crate::data::Trace;
use plotlib::repr::Plot;
use plotlib::style::{PointStyle, PointMarker, LineStyle};
use plotlib::view::ContinuousView;

pub const COLORS : [&'static str; 5] = [
    "#C1EBE1", "#90B9A9", "#FF0000", "#00FF00", "#0000FF"
];

impl Trace {
    pub fn lb_legend(&self) -> String {
        self.name.as_ref().map_or("Lower Bound".to_string(), |name| {
            name.to_owned() + " - Lower Bound"
        })
    }
    pub fn ub_legend(&self) -> String {
        self.name.as_ref().map_or("Upper Bound".to_string(), |name| {
            name.to_owned() + " - Upper Bound"
        })
    }
    pub fn fsz_legend(&self) -> String {
        self.name.as_ref().map_or("Frontier Size".to_string(), |name| {
            name.to_owned() + " - Frontier Size"
        })
    }

    pub fn lb_plot(&self, color: &str) -> Plot {
        Plot::new(self.lb_explored())
            .legend(self.lb_legend())
            .point_style(PointStyle::new().marker(PointMarker::Circle).colour(color))
            .line_style(LineStyle::new().colour(color))
    }
    pub fn ub_plot(&self, color: &str) -> Plot {
        Plot::new(self.ub_explored())
            .legend(self.ub_legend())
            .point_style(PointStyle::new().marker(PointMarker::Cross).colour(color))
            .line_style(LineStyle::new().colour(color))
    }
    pub fn fsz_plot(&self, color: &str) -> Plot {
        Plot::new(self.fringe_explored())
            .legend(self.fsz_legend())
            .point_style(PointStyle::new().marker(PointMarker::Square).colour(color))
            .line_style(LineStyle::new().colour(color))
    }
}

pub fn bounds_view(traces: &Vec<Trace>) -> ContinuousView {
    let mut view = ContinuousView::new()
        .x_label("Explored Nodes");

    for (i, trace) in traces.iter().enumerate() {
        let color = COLORS[i % COLORS.len()];
        view = view
            .add(trace.lb_plot(color))
            .add(trace.ub_plot(color));
    }

    view
}
pub fn fringe_view(traces: &Vec<Trace>) -> ContinuousView {
    let mut view = ContinuousView::new()
        .x_label("Explored Nodes");

    for (i, trace) in traces.iter().enumerate() {
        let color = COLORS[i % COLORS.len()];
        view = view
            .add(trace.fsz_plot(color));
    }

    view
}

fn min(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}
fn max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}