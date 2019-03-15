# 👥 Contributing Guidelines

Contributions are encouraged!
A good place to start is by looking at [good first issues](https://github.com/JSAbrahams/mamba/labels/good%20first%20issue).

## 😄 Etiquette

We want to maintain a pleasant atmosphere and encourage people to come up with new ideas.
We welcome a fresh perspective on ideas with open arms.

Please be respectful and mindful of one-another. 
This is an open source project, developers contribute to it in their free time.
Experience will vary greatly.

## 📝 Procedure

### ❗ Submitting an Issue

* Do check if there are already similar issues or pull requests before submitting a new issue
* Do reference issues in your issue if relevant
* Do use one of the templates

### ❓ Submitting a Pull Request

* Do check that there are no other pull requests implementing the same feature, fix or other
* Do reference the appropriate issues
* Do actively engage comments on the pull request
* Do use the template
* Do make sure the build passes, ideally by running `rustfmt` and `clippy` locally before pushing
* Do make sure that this PR is targeting one issue, large pull requests are not likely to get merged

In general, it is better to only comment on open pull requests and issues.

## 🔄 Continuous Integration Tools

We use continuous integration tools to help ensure that no regression takes place.
Please read the tooling section in the [readme](/README.md) to get more information on how to set up tooling locally, which are used by the CI tools.
A pull request will only be merged if all builds pass.
