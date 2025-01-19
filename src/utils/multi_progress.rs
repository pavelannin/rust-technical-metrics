use std::time::Duration;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub trait MultiProgressNew {
    fn add_with_style(&self, pb: ProgressBar, style: ProgressStyle) -> ProgressBar;
}

impl MultiProgressNew for MultiProgress {
    fn add_with_style(&self, pb: ProgressBar, style: ProgressStyle) -> ProgressBar {
        let pb = self.add(pb);
        pb.set_style(style);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }
}
