## Releases

Our release process is as follows:

1. In a new branch `release/vX.Y.Z`, update the [changelog](./CHANGELOG.md) to
   reflect and summarize all changes in the release. This involves:
   1. Running `unclog build -u` and copy pasting the output at the top of the
      `CHANGELOG.md` file, making sure to update the header with the new
      version.
   2. Running `unclog release --editor <editor> vX.Y.Z` to create a summary of
      all of the changes in this release.
      1. Your text editor will open. Write the release summary, and close the
         editor. Make sure to include a comment on whether or not the release
         contains consensus-breaking changes.
      2. Add this same summary to `CHANGELOG.md` as well.
   3. Committing the updated `CHANGELOG.md` file and `.changelog` directory to
      the repo.
2. Push this to a branch `release/vX.Y.Z` according to the version number of the
   anticipated release (e.g. `release/v0.18.0`) and open a **draft PR**.
3. If there were changes in the `ibc-derive` crate, we need to publish a new
   version of that crate.
   1. bump the version in `crates/ibc-derive/Cargo.toml`
   2. Publish `ibc-derive` with `cargo publish -p ibc-derive`
4. Bump the versions of all crates to the new version in their Cargo.toml and in
   the root `Cargo.toml` as well, and push these changes to the release PR.
   - If you released a new version of `ibc-derive` in step 3, make sure to
        update that dependency.
   - Verify that there is no dev-dependency among the workspace crates,
     except `ibc-testkit`. This is important, as `cargo-release` ignores
     dev-dependency edges. You may use `cargo-depgraph`:
      ```sh
      cargo depgraph --all-features --workspace-only --dev-deps | dot -Tpng > graph.png
      ```
     The command will generate a graph similar to this:
     ![alt test](docs/dev-deps-graph.png)
     The blue lines indicate dev dependencies; there should only be one blue line
     referring to the `ibc-testkit` dependency. So the above example would result
     in an unsuccessful release.
   - In order to resolve such a situation, the dev dependencies other than `ibc-testkit`
     can be manually released to crates.io first so that the subsequent crates that
     depend on them can then be released via the release process. For instructions
     on how to release a crate on crates.io, refer [here](https://doc.rust-lang.org/cargo/reference/publishing.html).
5. Run `cargo doc -p ibc --all-features --open` locally to double-check that all
   the documentation compiles and seems up-to-date and coherent. Fix any
   potential issues here and push them to the release PR.
6. Mark the PR as **Ready for Review** and incorporate feedback on the release.
   Once approved, merge the PR.
7. Checkout the `main` and pull it with `git checkout main && git pull origin main`.
8. Create a signed tag `git tag -s -a vX.Y.Z`. In the tag message, write the
   version and the link to the corresponding section of the changelog. Then push
   the tag to GitHub with `git push origin vX.Y.Z`.
   - The [release workflow][release.yaml] will run the `cargo release --execute`
   command in a CI worker.
9. If some crates have not been released, check the cause of the failure and
   act accordingly:
    1. In case of intermittent problems with the registry, try `cargo release`
      locally to publish any missing crates from this release. This step
      requires the appropriate privileges to push crates to [crates.io].
    2. If there is any new crate published locally, add
      [ibcbot](https://crates.io/users/ibcbot) to its owners list.
    3. In case problems arise from the source files, fix them, bump a new
      patch version (e.g. `v0.48.1`) and repeat the process with its
      corresponding new tag.
10. Once the tag is pushed, wait for the CI bot to create a GitHub release,
    then update the release description and append:
   `[📖CHANGELOG](https://github.com/cosmos/ibc-rs/blob/main/CHANGELOG.md#vXYZ)`

### Communications (non-technical) release pipeline

- Notify the communications team about the pending release and prepare an announcement.
- Coordinate with other organizations that are active in IBC development (e.g., Interchain) and keep them in the loop.

All done! 🎉

[crates.io]: https://crates.io
[release.yaml]: https://github.com/cosmos/ibc-rs/blob/main/.github/workflows/release.yaml
