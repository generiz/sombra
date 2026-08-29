use crate::{bundle::Priority, transport::LinkMetrics, TransportKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicy {
    pub congestion_weight: f32,
    pub energy_weight: f32,
    pub latency_weight: f32,
    pub delivery_weight: f32,
    pub metadata_weight: f32,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        Self {
            congestion_weight: 0.24,
            energy_weight: 0.16,
            latency_weight: 0.14,
            delivery_weight: 0.30,
            metadata_weight: 0.16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub transport: TransportKind,
    pub score: f32,
    pub reason: String,
}

impl RoutingPolicy {
    pub fn choose(&self, links: &[LinkMetrics], priority: Priority) -> Option<RoutingDecision> {
        links
            .iter()
            .copied()
            .filter(|link| link.available)
            .map(LinkMetrics::normalized)
            .map(|link| {
                let latency_penalty = latency_penalty(link.latency_ms, priority);
                let delivery_bonus = link.delivery_probability * self.delivery_weight;
                let score = delivery_bonus
                    - link.congestion * self.congestion_weight
                    - link.energy_cost * self.energy_weight
                    - latency_penalty * self.latency_weight
                    - link.metadata_exposure * self.metadata_weight;

                RoutingDecision {
                    transport: link.transport,
                    score,
                    reason: format!(
                        "delivery={:.2}, congestion={:.2}, energy={:.2}, latency={}ms, metadata={:.2}",
                        link.delivery_probability,
                        link.congestion,
                        link.energy_cost,
                        link.latency_ms,
                        link.metadata_exposure
                    ),
                }
            })
            .max_by(|a, b| a.score.total_cmp(&b.score))
    }
}

fn latency_penalty(latency_ms: u64, priority: Priority) -> f32 {
    let scale = match priority {
        Priority::Routine => 60_000.0,
        Priority::Important => 15_000.0,
        Priority::Urgent => 4_000.0,
    };
    (latency_ms as f32 / scale).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_reliable_low_congestion_link() {
        let policy = RoutingPolicy::default();
        let links = [
            LinkMetrics {
                transport: TransportKind::Internet,
                available: true,
                congestion: 0.90,
                energy_cost: 0.20,
                latency_ms: 120,
                delivery_probability: 0.70,
                metadata_exposure: 0.80,
            },
            LinkMetrics {
                transport: TransportKind::LongRange,
                available: true,
                congestion: 0.15,
                energy_cost: 0.50,
                latency_ms: 950,
                delivery_probability: 0.92,
                metadata_exposure: 0.35,
            },
        ];

        let decision = policy.choose(&links, Priority::Important).unwrap();
        assert_eq!(decision.transport, TransportKind::LongRange);
    }
}
