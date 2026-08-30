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

Radio drivers remain behind transport interfaces so hardware-specific work does not leak into the core protocol.

## v0.2

Sombra now includes a durable delay-tolerant queue in addition to the routing model and simulator.

Implemented:

- Rust core
- randomized collision-resistant bundle identifiers
- hop-limited bundles with TTL and priority
- bounded deduplication cache
- persistent local bundle store
- priority-aware scheduler
- expiry pruning
- bounded retry backoff
- adaptive transport scoring
- delivery probability, congestion, latency, energy and metadata-exposure metrics
- reproducible outage simulation
- JSON scenarios and reports
- terminal workflow for the local DTN queue
- cross-platform CI on Linux, Windows and macOS

## Simulation

```bash
cargo run -- simulate
```

Machine-readable output:

```bash
cargo run -- simulate --json
```

Custom scenario:

```bash
cargo run -- simulate --scenario examples/outage.json --json
```

## Durable queue

The queue stores opaque envelope bytes without inspecting message content. Authentication and encryption of those envelope bytes are deliberately a separate protocol layer and are not yet claimed as implemented.

Queue an already-prepared envelope:

```bash
cargo run -- queue enqueue \
  --envelope message.bin \
  --priority important \
  --ttl-secs 86400 \
  --hop-limit 8
```

Inspect bundles currently eligible for transmission:

```bash
cargo run -- queue next
```

Record a successful delivery:

```bash
cargo run -- queue attempt --id <bundle-id> --delivered
```

Record a failed attempt and schedule retry with bounded exponential backoff:

```bash
cargo run -- queue attempt --id <bundle-id>
```

Remove expired bundles:

```bash
cargo run -- queue prune
```

The default local store is `sombra-store.json` and is ignored by Git.

## Queue behavior

A node does not treat every bundle equally. Ready bundles are ordered by:

1. priority: urgent, important, routine
2. fewer previous failed attempts
3. oldest creation time
4. stable bundle ID ordering

Failed transmissions are deferred rather than immediately retried in a tight loop. The current research implementation starts at a 2 second retry delay and caps backoff at 5 minutes.

The persistent store is bounded. When full, new bundles are rejected instead of silently evicting existing data.

## Design direction

The architecture separates five concerns:

1. local identity and authenticated message envelopes
2. transport-independent bundles with TTL and deduplication
3. durable store-and-forward scheduling
4. a policy engine that selects among available transports
5. adapters for short-range radio, long-range low-bandwidth radio, intermittent IP and delay-tolerant carriage

The next protocol milestone is the authenticated envelope and local identity layer. Post-quantum hybrid key establishment remains a roadmap item and is not presented as implemented.

## Security position

Sombra aims to reduce unnecessary metadata and avoid central account infrastructure. It does not promise anonymity, invisibility or immunity to radio interference. A compromised endpoint can expose the messages visible on that endpoint. Radio range depends heavily on terrain, antennas, regulation, interference and hardware.

The v0.2 bundle store does not itself encrypt stored envelope bytes. Applications must not assume confidentiality at rest until the authenticated envelope layer is implemented and reviewed.

The project does not implement steganographic cover traffic or mechanisms intended to deceive monitoring systems.

See `docs/ARCHITECTURE.md`, `docs/THREAT_MODEL.md` and `docs/ROADMAP.md`.
