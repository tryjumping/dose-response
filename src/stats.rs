use std::collections::VecDeque;

use time::Duration;

#[derive(Debug)]
pub struct FrameStats {
    pub update: Duration,
    pub drawcalls: Duration,
}

#[derive(Debug)]
pub struct Stats {
    size: usize,
    frame_stats: VecDeque<FrameStats>,
    longest_update: Duration,
    longest_drawcalls: Duration,
}

impl Stats {
    pub fn new(size: usize) -> Self {
        Stats {
            size: size,
            frame_stats: VecDeque::with_capacity(size),
            longest_update: Duration::seconds(0),
            longest_drawcalls: Duration::seconds(0),
        }
    }

    pub fn push(&mut self, frame_stats: FrameStats) {
        if self.frame_stats.len() == self.size {
            self.frame_stats.pop_front();
        }

        if frame_stats.update > self.longest_update {
            self.longest_update = frame_stats.update;
        }
        if frame_stats.drawcalls > self.longest_drawcalls {
            self.longest_drawcalls = frame_stats.drawcalls;
        }
        self.frame_stats.push_back(frame_stats);
    }

    pub fn last_frames(&self, count: usize) -> FrameStatsIterator {
        let size = if count > self.frame_stats.len() {
            self.frame_stats.len()
        } else {
            count
        };
        FrameStatsIterator {
            frame_stats: &self.frame_stats,
            count: 0,
            size: size,
        }
    }

    pub fn longest_update(&self) -> Duration {
        self.longest_update
    }

    pub fn longest_drawcalls(&self) -> Duration {
        self.longest_drawcalls
    }
}

pub struct FrameStatsIterator<'a> {
    frame_stats: &'a VecDeque<FrameStats>,
    count: usize,  // starts at zero, goes up until size-1
    size: usize,  // the final number of items the iterator produces
}

impl<'a> Iterator for FrameStatsIterator<'a> {
    type Item = &'a FrameStats;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(self.frame_stats.len() >= self.size);

        if self.count < self.size {
            let last_index = self.frame_stats.len() - 1;
            let count = self.count;
            self.count += 1;
            Some(&self.frame_stats[last_index - count])
        } else {
            None
        }
    }
}
