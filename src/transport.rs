use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    ShortRange,
    LongRange,
    DelayTolerant,
    Internet,
}

impl TransportKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ShortRange => "short-range",
            Self::LongRange => "long-range",
            Self::DelayTolerant => "delay-tolerant",
            Self::Internet => "internet",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinkMetrics {
    pub transport: TransportKind,
    pub available: bool,
    pub congestion: f32,
    pub energy_cost: f32,
    pub latency_ms: u64,
    pub delivery_probability: f32,
    pub metadata_exposure: f32,
}

impl LinkMetrics {
    pub fn normalized(mut self) -> Self {
        self.congestion = self.congestion.clamp(0.0, 1.0);
        self.energy_cost = self.energy_cost.clamp(0.0, 1.0);
        self.delivery_probability = self.delivery_probability.clamp(0.0, 1.0);
        self.metadata_exposure = self.metadata_exposure.clamp(0.0, 1.0);
        self
    }
}
