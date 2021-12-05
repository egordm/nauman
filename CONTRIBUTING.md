# Contributing

## Getting started

As this is a hobby project, contributions are very welcome!

The easiest way for you to contribute right now is to use nauman, and see where it's lacking.

If you have a use case nauman does not cover, please file an issue. This is immensely useful to me, to anyone wanting to contribute to the project, and to you as well if the feature is implemented.

If you're interested in helping fix an [existing issue](https://github.com/EgorDm/nauman/issues), or an issue you just filed, help is appreciated.

## Project Structure

TODO

## Pull requests

Pull requests are _the_ way to change code using git. If you aren't familiar with them in general, GitHub has some [excellent documentation](https://help.github.com/articles/about-pull-requests/).

There aren't many hard guidelines in this repository on how specifically to format your request. Main points:

- Please include a descriptive title for your pull request, and elaborate on what's changed in the description.
- Feel free to open a PR before the feature is completely ready, and commit directly to the PR branch.
    - This is also great for review of PRs before merging
    - All commits will be squashed together on merge, so don't worry about force pushing yourself.
- Please include at least a short description in each commit, and more of one in the "main" feature commit. Doesn't
  have to be much, but someone reading the history should easily tell what's different now from before.
- If you have `rustfmt-nightly` installed, using it is recommended. I can also format the code after merging the code,
  but formatting it consistently will make reviewing nicer.

## Testing

Building nauman is as easy as is expected, `cargo build`.

To run and test the example programs, use:

```shell
cargo test
```

Feel free to add tests and examples demonstrating new features as you see fit. Pull requests which solely add new/interesting example jobs are also welcome.
