# Open Rust Coding Agent

 Open Rust Coding Agent for containerized, automated software engineering, built as part of a student team project at University of Tübingen.

## Quickstart

- Install the [minionrt CLI](https://github.com/minionrt/cli).
- Clone this repository on your machine:
  ```console
  git clone git@github.com:minionrt/orca.git
  ```
- From the root of this repository, run:
  ```console
  minion run --containerfile ./Containerfile
  ```
  The minionrt CLI will build a container image from the current state of your local clone of the `orca` repository.
  This container image will then subsequently be used to run the agent on the git repository in your current working directory.

## Acknowledgements

We would like to thank [Cohere](https://cohere.com/) for supporting this project:

This work was supported by compute credits from a Cohere Labs Research Grant, these grants are designed to support academic partners conducting research with the goal of releasing scientific artifacts and data for good projects.

## License

This project is distributed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
