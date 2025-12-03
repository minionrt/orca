# Open Rust Coding Agent

## Quickstart

- Install the [minionrt CLI](https://github.com/minionrt/cli).
- Clone this repository on your machine:
  ```console
  git clone git@github.com:orca-agent/orca.git
  ```
- From the root of this repository, run:
  ```console
  minion run --containerfile ./Containerfile
  ```
  The minionrt CLI will build a container image from the current state of your local clone of the `orca` repository.
  This container image will then subsequently be used to run the agent on the git repository in your current working directory.

## License

This project is distributed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
