//! Tiny tween system: a `Channel` is one animated float with a queue of segments.

use std::collections::VecDeque;
use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub enum Ease {
    SineInOut,
    QuadIn,
    QuadOut,
    BackOut,
}

impl Ease {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Ease::SineInOut => -((PI * t).cos() - 1.0) / 2.0,
            Ease::QuadIn => t * t,
            Ease::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let u = t - 1.0;
                1.0 + c3 * u * u * u + c1 * u * u
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Seg {
    to: f32,
    dur: f32,
    delay: f32,
    ease: Ease,
}

#[derive(Clone, Copy)]
struct Run {
    from: f32,
    elapsed: f32,
    seg: Seg,
}

#[derive(Clone)]
pub struct Channel {
    pub v: f32,
    cur: Option<Run>,
    queue: VecDeque<Seg>,
}

impl Channel {
    pub fn new(v: f32) -> Self {
        Self { v, cur: None, queue: VecDeque::new() }
    }

    /// Stop whatever is running; the value stays where it is.
    pub fn clear(&mut self) {
        self.cur = None;
        self.queue.clear();
    }

    pub fn set(&mut self, v: f32) {
        self.clear();
        self.v = v;
    }

    /// Append a segment to the sequence.
    pub fn push(&mut self, to: f32, dur: f32, delay: f32, ease: Ease) {
        self.queue.push_back(Seg { to, dur, delay, ease });
    }

    /// Replace the running animation with a single segment.
    pub fn go(&mut self, to: f32, dur: f32, delay: f32, ease: Ease) {
        self.clear();
        self.push(to, dur, delay, ease);
    }

    pub fn update(&mut self, mut dt: f32) {
        loop {
            if self.cur.is_none() {
                match self.queue.pop_front() {
                    Some(seg) => self.cur = Some(Run { from: self.v, elapsed: 0.0, seg }),
                    None => return,
                }
            }
            let mut run = self.cur.unwrap();
            run.elapsed += dt;
            let total = run.seg.delay + run.seg.dur;
            if run.elapsed >= total {
                dt = run.elapsed - total;
                self.v = run.seg.to;
                self.cur = None;
                continue;
            }
            let t = ((run.elapsed - run.seg.delay) / run.seg.dur.max(1e-6)).clamp(0.0, 1.0);
            self.v = run.from + (run.seg.to - run.from) * run.seg.ease.apply(t);
            self.cur = Some(run);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_reaches_target() {
        let mut c = Channel::new(0.0);
        c.go(1.0, 0.5, 0.1, Ease::QuadOut);
        for _ in 0..100 {
            c.update(0.01);
        }
        assert!((c.v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sequence_runs_in_order() {
        let mut c = Channel::new(0.0);
        c.push(1.0, 0.1, 0.0, Ease::SineInOut);
        c.push(-1.0, 0.1, 0.0, Ease::SineInOut);
        c.update(0.1);
        assert!((c.v - 1.0).abs() < 1e-4);
        c.update(0.1);
        assert!((c.v + 1.0).abs() < 1e-4);
    }
}
