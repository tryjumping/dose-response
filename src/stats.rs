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
    longest_updates: Vec<Duration>,
    longest_drawcalls: Vec<Duration>,
}

impl Stats {
    pub fn new(size: usize) -> Self {
        Stats {
            size: size,
            frame_stats: VecDeque::with_capacity(size),
            longest_updates: Vec::with_capacity(100),
            longest_drawcalls: Vec::with_capacity(100),
        }
    }

    pub fn push(&mut self, frame_stats: FrameStats) {
        if self.frame_stats.len() == self.size {
            self.frame_stats.pop_front();
        }

        self.longest_updates.sort();
        if self.longest_updates.capacity() == self.longest_updates.len() {
            // Since the vec is sorted, this is the lowest value:
            if self.longest_updates[0] < frame_stats.update {
                self.longest_updates[0] = frame_stats.update
            }
        } else {
            self.longest_updates.push(frame_stats.update);
        }

        self.longest_drawcalls.sort();
        if self.longest_drawcalls.capacity() == self.longest_drawcalls.len() {
            // Since the vec is sorted, this is the lowest value:
            if self.longest_drawcalls[0] < frame_stats.drawcalls {
                self.longest_drawcalls[0] = frame_stats.drawcalls
            }
        } else {
            self.longest_drawcalls.push(frame_stats.drawcalls);
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
        self.longest_updates.last().cloned().unwrap_or(Duration::seconds(0))
    }

    pub fn longest_drawcalls(&self) -> Duration {
        self.longest_drawcalls.last().cloned().unwrap_or(Duration::seconds(0))
    }

    pub fn mean_update(&self) -> f32 {
        self.frame_stats.iter()
            .map(|fs| fs.update.num_milliseconds() as f32)
            .fold(0.0, |acc, dur| acc + dur) / (self.frame_stats.len() as f32)
    }

    pub fn mean_drawcalls(&self) -> f32 {
        self.frame_stats.iter()
            .map(|fs| fs.drawcalls.num_milliseconds() as f32)
            .fold(0.0, |acc, dur| acc + dur) / (self.frame_stats.len() as f32)
    }

    pub fn longest_update_durations(&self) -> &[Duration] {
        &self.longest_updates
    }

    pub fn longest_drawcall_durations(&self) -> &[Duration] {
        &self.longest_drawcalls
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
