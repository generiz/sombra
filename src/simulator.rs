use crate::transport::LinkMetrics;
use crate::{Priority, RoutingPolicy, TransportKind};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub nodes: usize,
    pub messages: usize,
    pub infrastructure_outage: f32,
    pub short_range_density: f32,
    pub long_range_coverage: f32,
    pub seed: u64,
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            nodes: 120,
            messages: 10_000,
            infrastructure_outage: 0.83,
            short_range_density: 0.58,
            long_range_coverage: 0.64,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub nodes: usize,
    pub messages: usize,
    pub delivered: usize,
    pub delivery_ratio: f32,
    pub mean_latency_ms: u64,
    pub by_transport: BTreeMap<String, usize>,
}

#[derive(Default)]
pub struct Simulator {
    policy: RoutingPolicy,
}

impl Simulator {
    pub fn run(&self, scenario: &Scenario) -> SimulationReport {
        let mut rng = StdRng::seed_from_u64(scenario.seed);
        let mut delivered = 0usize;
        let mut latency_total = 0u128;
        let mut by_transport = BTreeMap::new();

        for _ in 0..scenario.messages {
            let links = generate_links(scenario, &mut rng);
            let priority = match rng.gen_range(0..100) {
                0..=4 => Priority::Urgent,
                5..=24 => Priority::Important,
                _ => Priority::Routine,
            };

            if let Some(decision) = self.policy.choose(&links, priority) {
                if let Some(link) = links.iter().find(|l| l.transport == decision.transport) {
                    if rng.gen::<f32>() <= link.delivery_probability {
                        delivered += 1;
                        latency_total += simulated_latency(link, &mut rng) as u128;
                        *by_transport
                            .entry(link.transport.label().to_string())
                            .or_insert(0) += 1;
                    }
                }
            }
        }

        let mean_latency_ms = if delivered == 0 {
            0
        } else {
            (latency_total / delivered as u128) as u64
        };

        SimulationReport {
            nodes: scenario.nodes,
            messages: scenario.messages,
            delivered,
            delivery_ratio: delivered as f32 / scenario.messages.max(1) as f32,
            mean_latency_ms,
            by_transport,
        }
    }
}

fn generate_links(s: &Scenario, rng: &mut StdRng) -> Vec<LinkMetrics> {
    let internet_available = rng.gen::<f32>() > s.infrastructure_outage.clamp(0.0, 1.0);
    let short_available = rng.gen::<f32>() < s.short_range_density.clamp(0.0, 1.0);
    let long_available = rng.gen::<f32>() < s.long_range_coverage.clamp(0.0, 1.0);

    vec![
        LinkMetrics {
            transport: TransportKind::ShortRange,
            available: short_available,
            congestion: rng.gen_range(0.10..0.85),
            energy_cost: rng.gen_range(0.10..0.35),
            latency_ms: rng.gen_range(40..350),
            delivery_probability: if short_available {
                rng.gen_range(0.72..0.97)
            } else {
                0.0
            },
            metadata_exposure: rng.gen_range(0.20..0.55),
        },
        LinkMetrics {
            transport: TransportKind::LongRange,
            available: long_available,
            congestion: rng.gen_range(0.05..0.60),
            energy_cost: rng.gen_range(0.35..0.70),
            latency_ms: rng.gen_range(350..2_500),
            delivery_probability: if long_available {
                rng.gen_range(0.68..0.94)
            } else {
                0.0
            },
            metadata_exposure: rng.gen_range(0.20..0.50),
        },
        LinkMetrics {
            transport: TransportKind::DelayTolerant,
            available: true,
            congestion: rng.gen_range(0.0..0.25),
            energy_cost: rng.gen_range(0.05..0.25),
            latency_ms: rng.gen_range(30_000..900_000),
            delivery_probability: rng.gen_range(0.75..0.98),
            metadata_exposure: rng.gen_range(0.10..0.35),
        },
        LinkMetrics {
            transport: TransportKind::Internet,
            available: internet_available,
            congestion: rng.gen_range(0.05..0.90),
            energy_cost: rng.gen_range(0.10..0.30),
            latency_ms: rng.gen_range(30..220),
            delivery_probability: if internet_available {
                rng.gen_range(0.88..0.995)
            } else {
                0.0
            },
            metadata_exposure: rng.gen_range(0.55..0.90),
        },
    ]
}

fn simulated_latency(link: &LinkMetrics, rng: &mut StdRng) -> u64 {
    let jitter = rng.gen_range(0.85..1.25);
    (link.latency_ms as f32 * jitter) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_survives_major_infrastructure_outage() {
        let scenario = Scenario {
            messages: 2_000,
            infrastructure_outage: 0.95,
            ..Scenario::default()
        };
        let report = Simulator::default().run(&scenario);
        assert!(report.delivery_ratio > 0.45);
        assert!(report.by_transport.keys().any(|k| k != "internet"));
    }

    #[test]
    fn simulation_is_reproducible() {
        let scenario = Scenario {
            messages: 500,
            ..Scenario::default()
        };
        let a = Simulator::default().run(&scenario);
        let b = Simulator::default().run(&scenario);
        assert_eq!(a.delivered, b.delivered);
        assert_eq!(a.mean_latency_ms, b.mean_latency_ms);
        assert_eq!(a.by_transport, b.by_transport);
    }
}
