# aw-watcher-network

ActivityWatch watcher that emits heartbeat events with network context.

## Data

Each heartbeat includes:

```json
{
  "public_ip": "145.x.x.x",
  "gateway_ip": "192.168.1.1",
  "connection_type": "lan",
  "location": "home",
  "title": "home"
}
```

## Usage

```sh
cargo run -- --testing
```

Flags:

- `--testing` send events to a testing bucket
- `--port <port>` override ActivityWatch server port (default: 5600)
- `--interval <seconds>` heartbeat interval (default: 60)
- `--config <path>` YAML config for location matching

## Location config

```yaml
locations:
  home:
    - 145.10.0.0/16
  office:
    - 83.160.45.12
```
