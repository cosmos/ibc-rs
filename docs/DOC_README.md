
# Updating the Docs

If you want to open a PR in IBC-rs to update the documentation, please follow
the guidelines in
[`CONTRIBUTING.md`](https://github.com/cosmos/ibc-rs/tree/main/CONTRIBUTING.md#updating-documentation)
and the [Documentation Writing Guidelines](./GUIDELINES.md).

## Stack

The documentation for IBC-rs is hosted at <https://ibc-rs.informal.systems/> and
built from the files in the `/docs` directory. It is built using the following
stack:

* [Docusaurus 2](https://docusaurus.io)
* [Algolia DocSearch](https://docsearch.algolia.com/)

  ```js
      algolia: {
        appId: "todo", 
        apiKey: "todo", 
        indexName: "todo", 
        contextualSearch: false,
      },
  ```

* GitHub Pages

## Docs Build Workflow

The docs are built and deployed automatically on GitHub Pages by a [GitHub
Action workflow](../.github/workflows/deploy-docs.yml). The workflow is
triggered on every push to the `main` and `release/v**` branches, every time
documentations or specs are modified.

### How It Works

There is a GitHub Action listening for changes in the `/docs` directory for the
`main` branch and each supported version branch (e.g. `release/v0.42.x`). Any
updates to files in the `/docs` directory will automatically trigger a website
deployment.

## How to Build the Docs Locally

Go to the `docs` directory and run the following commands. You need to have
Node >= 16.x and npm >= 8.5 installed.

```shell
cd docs
npm install
```

For starting only the current documentation, run:

```shell
npm run start
```

It runs `pre.sh` scripts to get all the docs that are not already in the
`docs/docs` folder. It also runs `post.sh` scripts to clean up the docs and
remove unnecessary files when quitting.

To build all the docs (including versioned documentation), run:

```shell
./build.sh
```

## Acknowledgements

This documentation includes codes and contents that was adapted from the
following sources:

* [Cosmos SDK](https://github.com/cosmos/cosmos-sdk): Portions of Docusaurus
  config and docs were used with modifications.

* [IBC-Go](https://github.com/cosmos/ibc-go): Portions of documentation markdown
  files were used with modifications.

We thank the authors of these projects for their valuable contribution, which
helped us in creating our own documentation.
