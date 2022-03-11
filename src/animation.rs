use std::marker::PhantomData;
use std::ops::Range;

pub struct Animation<'a, T, const N: usize> {
    stages: [AnimationStage<'a, T>; N],
}

impl<'a, T, const N: usize> Animation<'a, T, N> {
    pub fn new(stages: [AnimationStage<'a, T>; N]) -> Self {
        Self { stages }
    }

    pub fn run(&self, time: f32, input: T) {
        if let Some(stage) = self.stages.iter().find(|s| s.range().contains(&time)) {
            stage.execute(time, input);
        }
    }
}

pub struct AnimationStage<'a, T> {
    range: Range<f32>,
    interpolation: &'a dyn Fn(f32) -> f32,
    operation: &'a dyn Fn(f32, T),
    _marker: PhantomData<T>,
}

impl<'a, T> AnimationStage<'a, T> {
    pub fn new(
        range: Range<f32>,
        interpolation: &'a dyn Fn(f32) -> f32,
        operation: &'a dyn Fn(f32, T),
    ) -> Self {
        Self {
            range,
            interpolation,
            operation,
            _marker: PhantomData,
        }
    }

    fn range(&self) -> Range<f32> {
        self.range.clone()
    }

    fn execute(&self, time: f32, input: T) {
        (self.operation)((self.interpolation)(time), input)
    }
}
