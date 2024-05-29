<div align="center">
    <h1>ibc-rs</h1>
</div>

![banner](docs/static/img/ibc-rs-cover.svg)

<div align="center">
<div>

[![Cosmos ecosystem][cosmos-shield]][cosmos-link]

</div>
<div>

[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Code Coverage][codecov-image]][codecov-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Version][crates-io-version]
![Downloads][crates-io-downloads]
![Rust Stable][msrv-image]

</div>
</div>

Rust implementation of the Inter-Blockchain Communication (IBC) protocol that
hosts all the data structures and on-chain logic implementations of various IBC
core, clients and applications. This repository organized as a collection of
sub-crates that can be used independently or together.

## Project Structure

| <div style="width:100px">Crate</div>  | Description |
| ------------------------------------- | ----------- |
|[ibc](ibc)                             | Re-exports all the data structures and on-chain logic of various IBC core, clients and applications. |
|[ibc-data-types](ibc-data-types)       | Re-exports all the IBC data types that are shared across different IBC implementations. |
|[ibc-core](ibc-core)                   | Contains data structures and implementations of all the IBC core specifications. |
|[ibc-clients](ibc-clients)             | Contains data structures and implementations of various IBC light clients. |
|[ibc-apps](ibc-apps)                   | Contains data structures and implementations of various IBC applications. |
|[ibc-testkit](ibc-testkit)             | Provides testing toolkit to aid `ibc-rs` and host chains in writing integration tests. |
|[ibc-query](ibc-query)                 | Contains utility traits and implementations for querying states of an integrated IBC module. |
|[ibc-derive](ibc-derive)               | Derive macros for `ClientState` and `ConsensusState` traits, reducing boilerplate. |

## Contributing

IBC is specified in English in the [cosmos/ibc repo][ibc]. Any protocol changes
or clarifications should be contributed there.

If you're interested in contributing, please take a look at the
[CONTRIBUTING](./CONTRIBUTING.md) guidelines. We welcome and appreciate
community contributions!

## Community calls

The IBC-rs community calls are now part of the IBC Core community calls. During
these calls, we update the community on our current direction and gather
feedback on what to work on next. The community calls are also a platform for
you to update everyone else on what you're working on and find opportunities to
collaborate.

Please join the [Google group](https://groups.google.com/g/ibc-community) to
receive a calendar invitation to the monthly meetings.

## Versioning

We follow [Semantic Versioning][semver], though APIs are still under active
development.

## Resources

- [IBC Website][ibc-homepage]
- [IBC Specification][ibc]
- [IBC Go implementation][ibc-go]
- [Protobuf definitions in Rust][ibc-proto-rs]

## License

Copyright © 2022 Informal Systems Inc. and ibc-rs authors.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
the files in this repository except in compliance with the License. You may
obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.

[//]: # (badges)
[docs-image]: https://docs.rs/ibc/badge.svg
[docs-link]: https://docs.rs/ibc/
[build-image]: https://github.com/cosmos/ibc-rs/workflows/Rust/badge.svg
[build-link]: https://github.com/cosmos/ibc-rs/actions?query=workflow%3ARust
[codecov-image]: https://codecov.io/gh/cosmos/ibc-rs/branch/main/graph/badge.svg?token=wUm2aLCOu
[codecov-link]: https://codecov.io/gh/cosmos/ibc-rs
[license-image]: https://img.shields.io/crates/l/ibc
[license-link]: https://github.com/cosmos/ibc-rs/blob/main/LICENSE
[crates-io-version]: https://img.shields.io/crates/v/ibc.svg
[crates-io-downloads]: https://img.shields.io/crates/d/ibc.svg
[msrv-image]: https://img.shields.io/crates/msrv/ibc

[//]: # (general links)
[ibc]: https://github.com/cosmos/ibc
[ibc-go]: https://github.com/cosmos/ibc-go
[ibc-proto-rs]: https://github.com/cosmos/ibc-proto-rs
[ibc-homepage]: https://cosmos.network/ibc
[cosmos-link]: https://cosmos.network
[semver]: https://semver.org/
[cosmos-shield]: https://img.shields.io/static/v1?label=&labelColor=1B1E36&color=1B1E36&message=cosmos%20ecosystem&style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDI0LjMuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNpb249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCAyNTAwIDI1MDAiIHN0eWxlPSJlbmFibGUtYmFja2dyb3VuZDpuZXcgMCAwIDI1MDAgMjUwMDsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgo8c3R5bGUgdHlwZT0idGV4dC9jc3MiPgoJLnN0MHtmaWxsOiM2RjczOTA7fQoJLnN0MXtmaWxsOiNCN0I5Qzg7fQo8L3N0eWxlPgo8cGF0aCBjbGFzcz0ic3QwIiBkPSJNMTI1Mi42LDE1OS41Yy0xMzQuOSwwLTI0NC4zLDQ4OS40LTI0NC4zLDEwOTMuMXMxMDkuNCwxMDkzLjEsMjQ0LjMsMTA5My4xczI0NC4zLTQ4OS40LDI0NC4zLTEwOTMuMQoJUzEzODcuNSwxNTkuNSwxMjUyLjYsMTU5LjV6IE0xMjY5LjQsMjI4NGMtMTUuNCwyMC42LTMwLjksNS4xLTMwLjksNS4xYy02Mi4xLTcyLTkzLjItMjA1LjgtOTMuMi0yMDUuOAoJYy0xMDguNy0zNDkuOC04Mi44LTExMDAuOC04Mi44LTExMDAuOGM1MS4xLTU5Ni4yLDE0NC03MzcuMSwxNzUuNi03NjguNGM2LjctNi42LDE3LjEtNy40LDI0LjctMmM0NS45LDMyLjUsODQuNCwxNjguNSw4NC40LDE2OC41CgljMTEzLjYsNDIxLjgsMTAzLjMsODE3LjksMTAzLjMsODE3LjljMTAuMywzNDQuNy01Ni45LDczMC41LTU2LjksNzMwLjVDMTM0MS45LDIyMjIuMiwxMjY5LjQsMjI4NCwxMjY5LjQsMjI4NHoiLz4KPHBhdGggY2xhc3M9InN0MCIgZD0iTTIyMDAuNyw3MDguNmMtNjcuMi0xMTcuMS01NDYuMSwzMS42LTEwNzAsMzMycy04OTMuNSw2MzguOS04MjYuMyw3NTUuOXM1NDYuMS0zMS42LDEwNzAtMzMyCglTMjI2Ny44LDgyNS42LDIyMDAuNyw3MDguNkwyMjAwLjcsNzA4LjZ6IE0zNjYuNCwxNzgwLjRjLTI1LjctMy4yLTE5LjktMjQuNC0xOS45LTI0LjRjMzEuNi04OS43LDEzMi0xODMuMiwxMzItMTgzLjIKCWMyNDkuNC0yNjguNCw5MTMuOC02MTkuNyw5MTMuOC02MTkuN2M1NDIuNS0yNTIuNCw3MTEuMS0yNDEuOCw3NTMuOC0yMzBjOS4xLDIuNSwxNSwxMS4yLDE0LDIwLjZjLTUuMSw1Ni0xMDQuMiwxNTctMTA0LjIsMTU3CgljLTMwOS4xLDMwOC42LTY1Ny44LDQ5Ni44LTY1Ny44LDQ5Ni44Yy0yOTMuOCwxODAuNS02NjEuOSwzMTQuMS02NjEuOSwzMTQuMUM0NTYsMTgxMi42LDM2Ni40LDE3ODAuNCwzNjYuNCwxNzgwLjRMMzY2LjQsMTc4MC40CglMMzY2LjQsMTc4MC40eiIvPgo8cGF0aCBjbGFzcz0ic3QwIiBkPSJNMjE5OC40LDE4MDAuNGM2Ny43LTExNi44LTMwMC45LTQ1Ni44LTgyMy03NTkuNVMzNzQuNCw1ODcuOCwzMDYuOCw3MDQuN3MzMDAuOSw0NTYuOCw4MjMuMyw3NTkuNQoJUzIxMzAuNywxOTE3LjQsMjE5OC40LDE4MDAuNHogTTM1MS42LDc0OS44Yy0xMC0yMy43LDExLjEtMjkuNCwxMS4xLTI5LjRjOTMuNS0xNy42LDIyNC43LDIyLjYsMjI0LjcsMjIuNgoJYzM1Ny4yLDgxLjMsOTk0LDQ4MC4yLDk5NCw0ODAuMmM0OTAuMywzNDMuMSw1NjUuNSw0OTQuMiw1NzYuOCw1MzcuMWMyLjQsOS4xLTIuMiwxOC42LTEwLjcsMjIuNGMtNTEuMSwyMy40LTE4OC4xLTExLjUtMTg4LjEtMTEuNQoJYy00MjIuMS0xMTMuMi03NTkuNi0zMjAuNS03NTkuNi0zMjAuNWMtMzAzLjMtMTYzLjYtNjAzLjItNDE1LjMtNjAzLjItNDE1LjNjLTIyNy45LTE5MS45LTI0NS0yODUuNC0yNDUtMjg1LjRMMzUxLjYsNzQ5Ljh6Ii8+CjxjaXJjbGUgY2xhc3M9InN0MSIgY3g9IjEyNTAiIGN5PSIxMjUwIiByPSIxMjguNiIvPgo8ZWxsaXBzZSBjbGFzcz0ic3QxIiBjeD0iMTc3Ny4zIiBjeT0iNzU2LjIiIHJ4PSI3NC42IiByeT0iNzcuMiIvPgo8ZWxsaXBzZSBjbGFzcz0ic3QxIiBjeD0iNTUzIiBjeT0iMTAxOC41IiByeD0iNzQuNiIgcnk9Ijc3LjIiLz4KPGVsbGlwc2UgY2xhc3M9InN0MSIgY3g9IjEwOTguMiIgY3k9IjE5NjUiIHJ4PSI3NC42IiByeT0iNzcuMiIvPgo8L3N2Zz4K
