pub struct RisingEdgeTrigger<I, T>
where
    I: Iterator<Item = T>,
    T: PartialOrd + Copy,
{
    iter: I,
    threshold: T,
    prev_value: Option<T>,
    triggered: bool,
}

impl<I, T> RisingEdgeTrigger<I, T>
where
    I: Iterator<Item = T>,
    T: PartialOrd + Copy,
{
    pub fn new(iter: I, threshold: T) -> Self {
        RisingEdgeTrigger {
            iter,
            threshold,
            prev_value: None,
            triggered: false,
        }
    }
}

impl<I, T> Iterator for RisingEdgeTrigger<I, T>
where
    I: Iterator<Item = T>,
    T: PartialOrd + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.triggered {
            return self.iter.next();
        }

        for current_value in self.iter.by_ref() {
            if let Some(prev_value) = self.prev_value {
                if current_value > self.threshold && prev_value <= self.threshold {
                    self.triggered = true;
                    self.prev_value = Some(current_value);
                    return Some(current_value);
                }
            }
            self.prev_value = Some(current_value);
        }

        None
    }
}

pub trait RisingEdgeTriggerExt<I, T>: Iterator<Item = T>
where
    I: Iterator<Item = T>,
    T: PartialOrd + Copy,
{
    fn rising_edge_trigger(self, threshold: T) -> RisingEdgeTrigger<I, T>;
}

impl<I, T> RisingEdgeTriggerExt<I, T> for I
where
    I: Iterator<Item = T>,
    T: PartialOrd + Copy,
{
    fn rising_edge_trigger(self, threshold: T) -> RisingEdgeTrigger<I, T> {
        RisingEdgeTrigger::new(self, threshold)
    }
}
