use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{sync::syncing::impl_serde::MakeSyncableValue, timer::Timer, utils::HSV};

pub trait Tweenable: Copy + core::fmt::Debug + Eq + PartialEq + 'static {
    fn step(from: &Self, to: &Self, step: f32) -> Self;
}

impl Tweenable for u8 {
    fn step(from: &Self, to: &Self, step: f32) -> Self {
        (((to - from) as f32 * step) as u8) + from
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TweenCurve {
    Linear,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TweenDirection {
    Forwards,
    Backwards,
}

impl TweenCurve {
    fn step(&self, start: u32, finish: u32) -> f32 {
        let now = Timer::read();

        if now < start {
            return 0.0;
        } else if now > finish {
            return 1.0;
        }

        match self {
            TweenCurve::Linear => (now - start) as f32 / (finish - start) as f32,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tween<Value: Tweenable> {
    pub duration: u32,
    direction: TweenDirection,
    from: Value,
    to: Value,
    start: u32,
    finish: u32,
    curve: TweenCurve,
}

impl<Value: Tweenable> MakeSyncableValue for Tween<Value> {}

impl<Value: Tweenable + Serialize + DeserializeOwned> Tween<Value> {
    pub fn new(from: Value, to: Value, duration: u32) -> Tween<Value> {
        let now = Timer::read();

        Tween {
            direction: TweenDirection::Forwards,
            duration,
            from,
            to,
            start: now,
            finish: now + duration,
            curve: TweenCurve::Linear,
        }
    }

    pub fn delay(mut self, delay: u32) -> Self {
        self.start = Timer::read() + delay;
        self.finish = self.start + self.duration;
        self
    }

    pub fn finished(&self) -> bool {
        Timer::read() > self.finish
    }

    pub fn next(&self) -> Value {
        let step_value = if matches!(self.direction, TweenDirection::Forwards) {
            self.curve.step(self.start, self.finish)
        } else {
            1.0 - self.curve.step(self.start, self.finish)
        };

        Value::step(&self.from, &self.to, step_value)
    }

    pub fn reset(&mut self) -> &mut Self {
        self.start = Timer::read();
        self.finish = self.start + self.duration;
        self
    }

    pub fn direction(mut self, direction: TweenDirection) -> Self {
        self.direction = direction;
        self
    }
}
