# Contributing

Thank you for your interest in contributing to ibc-rs!🎉

The rest of this document outlines the best practices for contributing to this
repository:

- [Decision Making](#decision-making) - process for agreeing to changes
- [Issues](#issues) - what makes a good issue
- [Pull Requests](#pull-requests) - what makes a good pull request
- [Forking](#forking) - fork the repo to make pull requests
- [Changelog](#changelog) - changes must be recorded in the changelog

## Decision Making

The following process leads to the best chance of landing the changes in `main`.

All new contributions should **start with a GitHub issue** which captures the
problem you're trying to solve. Starting off with an issue allows for early
feedback. See the [Issues](#issues) section for more details.

Once the issue is created, maintainers may request that more detailed
documentation be written in the form of a Request for Comment (RFC) or an
Architectural Decision Record
([ADR](https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/README.md)).

Discussion at the RFC stage will build collective understanding of the
dimensions of the problem and help structure conversations around trade-offs.

When the problem is well understood but the solution leads to large structural
changes to the code base, these changes should be proposed in the form of an
[Architectural Decision Record (ADR)](./docs/architecture/). The ADR will help
build consensus on an overall strategy to ensure that the code base maintains
coherence in the larger context. If you are not comfortable with writing an ADR,
you can open a regular issue and the maintainers will help you turn it into an
ADR.

When the problem and the proposed solution are well understood, changes should
start with a [draft pull
request](https://github.blog/2019-02-14-introducing-draft-pull-requests/)
against `main`. The draft status signals that work is underway. When the work is
ready for feedback, hitting "Ready for Review" will signal to the maintainers to
take a look.

Implementation trajectories should aim to proceed where possible as a series
of smaller incremental changes, in the form of small PRs that can be merged
quickly. This helps manage the load for reviewers and reduces the likelihood
that PRs will sit open for long periods of time.

![Contributing flow](https://github.com/tendermint/tendermint/blob/v0.33.6/docs/imgs/contributing.png?raw=true)

## Issues

We appreciate bug reports, feature requests, and contributions to our project.
If you're new to contributing, a great starting point is to explore [A:
good-first-issues](https://github.com/cosmos/ibc-rs/labels/A%3A%20good-first-issue)
on our GitHub repository. These are well-defined tasks ideal for developers who
are new to ibc-rs. When opening an issue, kindly adhere to the following
guidelines:

1. **Search existing issues**: Before opening a new issue, please search
   existing issues to ensure that is not a duplicates. If you would like to work
   on an issue which already exists please indicate so by leaving a comment. If
   what you'd like to work on hasn't already been covered by an issue, then open
   a new one to get the process going.

2. **Provide a clear and descriptive title**: This helps others understand the
   nature of the issue at a glance.

3. **Provide detailed information**: In the issue description, clearly state the
   purpose of the issue and follow the guidelines of the issue template

Note: A maintainer will take care of assigning the appropriate labels to your
issue with the following convention:

- Objective-level (WHY): conveys the overarching purpose or objective of the
  issue by labels starting with "O" like `O: security`, `O: new-feature`, etc.

- Scope-level (WHICH): specifies the part of the system that the issue pertains
  to and labels starting with "S" like `S: non-cosmos`, `S: no-std`, etc.

- Admin-level (HOW) includes relevant administrative considerations on how best
  handling the issue and labels starting with "A" like `A: help-wanted`, `A:
  critical`, etc.

If the issue was tagged `A: low-priority`, we'll do our best to review it in a
timely manner, but please expect longer wait times for a review in general. If a
low priority issue is important to you, please leave a comment explaining why,
and we will re-prioritize it!

## Pull Requests

Before making any changes, please create a new issue or refer to an existing one
to initiate a discussion with us. This allows for collaborative discussions on
the design and proposed implementation of your changes. This way, we ensure that
your efforts align with our objectives and increase the likelihood of your
contribution being accepted.

Pull requests should aim to be small and self-contained to facilitate quick
review and merging. Larger change sets should be broken up across multiple PRs.
Commits should be concise but informative, and moderately clean. Please follow
these guidelines when opening a pull request:

- If you have write access to the ibc-rs repo,  directly branch off from `HEAD`
  of `main`. Otherwise, check [Forking](#forking) section for instructions.

- Branch names should be prefixed with the convention
  `{moniker}/{issue#}-branch-name`.

- Ensure PR titles adhere to the format of `{type}: {description}`. The {type}
  should be selected from the following list:
  - feat: for feature work.
  - fix: for bug fixes.
  - imp: for refactors and improvements.
  - docs: for documentation changes.
  - test: for addition or improvements of unit, integration and e2e tests.
  - deps: for changes to dependencies.
  - chore: for any miscellaneous changes that don't fit into another category.
  - If any change is breaking, follow the format below: type + (api)! for api
    breaking changes, e.g. fix(api)!: api breaking fix type + (statemachine)!
    for state machine breaking changes, e.g. fix(statemachine)!: state machine
    breaking fix api breaking changes take precedence over statemachine breaking
    changes.

- Commit messages should follow the [Conventional Commits
  specification](https://www.conventionalcommits.org/en/v1.0.0/).

- Make reference to the relevant issue by including `Closes: #<issue number>` in
  the PR’s description to auto-close the related issue once the PR is merged.

- Update any relevant documentation and include tests.

- Add a corresponding entry in the `.changelog` directory using `unclog`. See
  the [Changelog](#changelog) section for more details.

- If possible, tick the `Allow edits from maintainers` box when opening your PR
  from your fork of ibc-rs. This allows us to directly make minor edits /
  refactors and speeds up the merging process.

## Forking

If you do not have write access to the repository, your contribution should be
made through a fork on Github. Fork the repository, contribute to your fork
(either in the `main` branch of the fork or in a separate branch), and then
make a pull request back upstream.

When forking, add your fork's URL as a new git remote in your local copy of the
repo. For instance, to create a fork and work on a branch of it:

- Create the fork on GitHub, using the fork button.
- `cd` to the original clone of the repo on your machine
- `git remote rename origin upstream`
- `git remote add origin git@github.com:<location of fork>`

Now `origin` refers to your fork and `upstream` refers to the original version.
Now `git push -u origin main` to update the fork, and make pull requests
against the original repo.

To pull in updates from the origin repo, run `git fetch upstream` followed by
`git rebase upstream/main` (or whatever branch you're working in).

## Changelog

Every non-trivial PR must update the [CHANGELOG](CHANGELOG.md). This is
accomplished indirectly by adding entries to the `.changelog` folder in
[`unclog`](https://github.com/informalsystems/unclog) format using the `unclog`
CLI tool. `CHANGELOG.md` will be built by whomever is responsible for performing
a release just prior to release - this is to avoid changelog conflicts prior to
releases.

### Install `unclog`

```bash
cargo install unclog
```

### Examples

Add a `.changelog` entry to signal that a bug was fixed, without mentioning any
component.

```bash
unclog add -i update-unclog-instructions -s bug -n 1634 -m "Update CONTRIBUTING.md for latest version of unclog" --editor vim
```

Add a .changelog entry for the `ibc` crate.

```bash
unclog add -c ibc -s features --id a-new-feature --issue-no 1235 -m "msg about this new-feature" --editor vim
```

### Preview unreleased changes

```bash
unclog build -u
```

The Changelog is *not* a record of what Pull Requests were merged; the commit
history already shows that. The Changelog is a notice to users about how their
expectations of the software should be modified. It is part of the UX of a
release and is a *critical* user facing integration point. The Changelog must be
clean, inviting, and readable, with concise, meaningful entries. Entries must be
semantically meaningful to users. If a change takes multiple Pull Requests to
complete, it should likely have only a single entry in the Changelog describing
the net effect to the user. Instead of linking PRs directly, we instead prefer
to log issues, which tend to be higher-level, hence more relevant for users.

When writing Changelog entries, ensure they are targeting users of the software,
not fellow developers. Developers have much more context and care about more
things than users do. Changelogs are for users.

Changelog structure is modeled after [Tendermint
Core](https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md) and
[Hashicorp Consul](http://github.com/hashicorp/consul/tree/main/CHANGELOG.md).
See those changelogs for examples.

We currently split changes for a given release between these four sections:
Breaking Changes, Features, Improvements, Bug Fixes.

Entries in the changelog should initially be logged in the **Unreleased**
section, which represents a "staging area" for accumulating all the changes
throughout a release (see [Pull Requests](#pull-requests) above). With each
release, the entries then move from this section into their permanent place
under a specific release number in Changelog.

Changelog entries should be formatted as follows:

```md
- [pkg] Some description about the change ([#xxx](https://github.com/cosmos/ibc-rs/issues/xxx)) (optional @contributor)
```

Here, `pkg` is the part of the code that changed (typically a top-level crate,
but could be `<crate>/<module>`), `xxx` is the issue number, and `contributor`
is the author/s of the change.

It's also acceptable for `xxx` to refer to the relevant pull request, but issue
numbers are preferred. Note this means issues (or pull-requests) should be
opened first so the changelog can then be updated with the corresponding number.

Changelog entries should be ordered alphabetically according to the `pkg`, and
numerically according to their issue/PR number.

Changes with multiple classifications should be doubly included (eg. a bug fix
that is also a breaking change should be recorded under both).

Breaking changes are further subdivided according to the APIs/users they impact.
Any change that effects multiple APIs/users should be recorded multiply - for
instance, a change to some core protocol data structure might need to be
reflected both as breaking the core protocol but also breaking any APIs where
core data structures are exposed.
