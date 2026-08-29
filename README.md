# Sombra

Sombra is a research prototype for resilient messaging across degraded, intermittent and off-grid networks.

It is not "WhatsApp without Internet." The design assumes that infrastructure can disappear for minutes or hours and that no single radio path should be treated as permanent.

> Sombra does not bet on one heroic hop. It bets on many ordinary ones.

## Three skins

Sombra models three communication layers and lets a local routing policy choose among the links that are actually available.

| Layer | Typical role | Candidate transports |
| --- | --- | --- |
| Short | Dense local environments | BLE, local Wi-Fi, Wi-Fi Aware |
| Medium | Community and regional relays | LoRa-class adapters and fixed relays |
| Long | Sparse or disconnected environments | delay-tolerant store-and-forward, physical carriage, intermittent IP bridges |

The current repository implements the routing model, message bundles, a deterministic simulator and failure scenarios. Radio drivers are deliberately kept behind transport interfaces so that hardware-specific work does not leak into the core protocol.

## What is implemented

- Rust core
- adaptive transport scoring
- delivery probability, congestion, latency, energy and metadata-exposure metrics
- hop-limited message bundles
- delay-tolerant transport model
- reproducible outage simulation
- JSON scenarios and reports
- cross-platform CI

## Run a simulation

```bash
cargo run -- simulate
```

Machine-readable output:

```bash
cargo run -- simulate --json
```

A custom scenario can be supplied as JSON:

```bash
cargo run -- simulate --scenario examples/outage.json --json
```

## Design direction

The long-term architecture separates four concerns:

1. local identity and authenticated message envelopes
2. transport-independent bundles with TTL and deduplication
3. a policy engine that selects among available transports
4. adapters for short-range radio, long-range low-bandwidth radio, intermittent IP and delay-tolerant carriage

Post-quantum hybrid key establishment is part of the protocol roadmap. It is not claimed as implemented until an audited, stable construction is integrated and tested.

## Security position

Sombra aims to reduce unnecessary metadata and avoid central account infrastructure. It does not promise anonymity, invisibility or immunity to radio interference. A compromised endpoint can expose the messages visible on that endpoint. Radio range depends heavily on terrain, antennas, regulation, interference and hardware.

The project does not implement steganographic cover traffic or mechanisms intended to deceive monitoring systems.

See `docs/ARCHITECTURE.md` and `docs/THREAT_MODEL.md`.
