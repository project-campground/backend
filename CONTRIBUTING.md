# Terms

- **Prohibited** -- strictest form of convention. Not following a guideline where something is prohibited may result in some kind of punishment, warning or for Campground developers who may have not known better, a notice.
- **Should** -- something that may result in a notice if sometimes done or unaccepted pull request in some cases.
- **Recommended** -- this may result in a notice, but generally this means that the convention is more free-form and if any other convention is used that is understandable enough, it may be ignored. Though, if it happens often a notice may be given.

Note that generally we believe most contributors want to help Campground and are not being annoying on purpose. If you are being unnecessarily annoying, stricter punishment may be done. Developers and counsellors have different agreements from contributors.

# Use of AI and Agents

Commits that have descriptions or messages and documentation written by agents or AI in general is strictly **_prohibited_** in this repository or any other Campground repository.

Code that was written by AI **_must_** be reviewed by a human and understandable to a human **_line-by-line_** and not have their knowledge come from prompt asking "What does this code do?". Any contributor, who is not an official developer of Campground, may be banned from contributing internally or have their pull requests dismissed without prior notice if the contributor is found to violate this section.

# Commit messages

As noted previously, commit messages must be written by a human.

Commit messages **should** use [Conventional Commit format](https://www.conventionalcommits.org/en/v1.0.0/).

As noted in the Conventional Commit standard, the commit format is as follows:

```
<type>(optional scope): <description>

[optional body]

[optional footer(s)]
```

> [!NOTE]
> Note the changes of scoping, where by Conventional Commit standard, scoping is inside the brackets (`[]`), whereas we use parenthesis scoping (`()`).

As an addition to `feat` and `fix` types outlined in the Conventional Commits, we have also the following types:

- `refactor` -- refactoring of pieces of code or the repository
- `revert` -- the reversion of a specific commit
- `test` -- inclusion of new tests, removal or changes to existing tests in the code
- `docs` -- changes to the documentation

## Repository scopes

As of now, there is only one scope: `appview`. This means that scoping is optional, but may be mandatory in the future for discovery back-ends and such. Whenever multiple scopes are modified, they must be listed in a comma. The following is an example of modification of code in 2 scopes:

```
docs(appview, discovery): explicitly state that exampleId in campsites is deprecated
```

## Commit descriptions

Generally commit descriptions are more free-form, but for `feat` commits, it is recommended to have a verb in Present-Tense form followed by a noun or a phrase in a comma list (though, not mandatory). Example:

```
feat(appview): add `exampleId` field to campsites, add `parentExampleId` to tents
```

This is also a recommended form for most other types, except `fix` and `revert` types. While the form is allowed for `fix`, it is more likely that you may need to instead list out **WHAT** has been modified (`fix(appview): getCampsite route giving empty 'campsites' array`), rather than what was done in the aforementioned recommended form (`fix(appview): add back campsites to 'campsites' array in getCampsite route`).

# Branches

Generally, the branches following this naming convention:

```
yyyy/branch-summary
```

Where yyyy represents the full-year when the branch was created and `branch-summary` as general summary of the features and changes done in the branch. It's generally recommended to keep the `branch-summary` max 5 words, but it's recommended to have 1-3 words. Developers and counsellors may create the branches.

Convention is ignored by `dev`, `main`, `qa` and `test`. Different guidelines apply to these branches:

- `main` represents prod-ready code that is ready for all users and should not be pushed to without prior notice
    - `README.md` or documentation changes may be included, as well as grammar/syntax/wording fixes may be done. Other changes to other Markdown files that are root-level should be done with the notice of others and is generally not recommended.
    - The direct commits and documentation changes should not be visible to regular users without going through the process.
- `qa` represents canary/beta version that is more stable, but may still result in significant bugs or instability before being merged into `main`. This may be published to the public
- `test` represents developer/alpha version that is more internally tested by select-few individuals/volunteers and may be more unstable than `qa` branch.
- `dev` represents inherently unstable version that is either the result of general merging of certain branches likely without extensive testing by anyone else other than the developers. For smaller changes, you can directly contribute to `dev`, but for bigger features and changes that may surpass 5 commits, consider creating a new feature branch(`yyyy/branch-summary`) and eventually a pull request that may be merged into `dev`.

As such, from this, a work-flow could be gathered:
`2026/example-branch` -> `dev` -> `test` -> `qa` -> `main`

Pull Requests may be merged into any of these branches.

Of course, this may not be followed by works when the branches are used for Pull Requests.

# New Features

New features added under pull requests by contributors need to be discussed prior and if the process is ignored, the pull requests may be closed with a reason stating why. Developers are generally notified of the features and branches that shall be done. Smaller features (like additional fields added to some kind of models) are allowed, but might take awhile to approve until it is discussed internally.

# Other root-level MD files

Modifying other Markdown files at the root level of the source code directly, especially without a Pull Request, **_excluding_** `README.md` and any Markdown files that are not in the root-level (those that are in one of the directories), **_IS PROHIBITED_** without prior notice of any of the counsellors.
