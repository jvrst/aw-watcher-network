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

TODO: add build scripts for Linux & Windows.

1. Run `./build.sh`; it should automatically detect your platform & run a build script.

The basic idea is to generate a release build & run the watcher with your platform's init service.


## Development

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
