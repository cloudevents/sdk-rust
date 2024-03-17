# How to create a Release

To create a new release, do the following:
- Bump the version in the README, lib.rs and cargo.toml
- Try to run `cargo test --all-features`, `cargo doc --all-features --lib` and
  `cargo publish --dry-run`
- If none of the above commands fail, PR the changes and merge it
- Checkout `main` on your local machine and run `cargo publish`
- Once that is done, create the release in the Github UI (make sure it
  creates the git tag as well) and that's it!
