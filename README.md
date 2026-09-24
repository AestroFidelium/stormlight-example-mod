# Stormlight example mod

[![CI](https://github.com/AestroFidelium/stormlight-example-mod/actions/workflows/ci.yml/badge.svg)](https://github.com/AestroFidelium/stormlight-example-mod/actions/workflows/ci.yml)

A minimal, complete gameplay mod for the Stormlight engine: one unit a player
can drive, with two abilities.

- **spark**: a skillshot. It spends energy and throws a missile that damages
  the first enemy it touches.
- **guard**: a self-cast. It spends energy and raises armor for four seconds.

It is a starting point to copy, and it is also the end-to-end check for the
public mod pipeline. CI builds it to wasm and loads it into the real engine host
([`modload`](https://github.com/AestroFidelium/stormlight-shared)), which runs its
registration exactly as a server would.

> This repository is a read-only mirror of a private upstream, where development
> and planning happen. Bug reports and feedback are welcome as
> [issues](https://github.com/AestroFidelium/stormlight-example-mod/issues); pull requests
> are disabled.

## Build and load

Needs stable Rust with the wasm target
(`rustup target add wasm32-unknown-unknown`).

```console
$ cargo nextest run          # content tests, natively
$ tools/package.sh           # → dist/example/{manifest.toml, *.wasm}
$ cargo install --git https://github.com/AestroFidelium/stormlight-shared.git stormlight_modloader --bin modload
$ modload dist/example
mod example v0.1.0 (Server)
  name:      Example
  abi:       0.1.0
  units:     1
  abilities: 2
  talents:   0
  buffs:     1
  curves:    0
  tags:      1 (1 class links)
```

`dist/example/` is a complete mod package. Zip its contents to get the `.zip`
form.

## Layout

| File | What it is |
| --- | --- |
| `src/lib.rs` | The whole mod: one `register_mod!` closure and three descriptor builders. |
| `manifest.toml` | Id, kind (`server` = gameplay), wasm entry, and the ABI version. The host reads it before it touches the wasm. |
| `tests/property/` | [bolero](https://github.com/camshaft/bolero) tests over the registration. Every bound ability exists and is affordable from a full pool, and every missile hits enemies only. |

The tests call `__stormlight_registration()`, the same builder the wasm export
runs, so a broken declaration fails in a named test instead of showing up as a
unit that silently does nothing.

## Working against a local SDK

The SDK is a git dependency. To build against a checkout next to this one:

```console
$ cargo build --config 'patch."https://github.com/AestroFidelium/stormlight-sdk.git".stormlight_mod_sdk.path="../sdk/pdk"'
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
