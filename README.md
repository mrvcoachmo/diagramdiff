# diagramdiff

diagramdiff shows what changed between two versions of an Excalidraw diagram, as boxes and arrows added and removed instead of lines of JSON. Excalidraw is the only format so far, and draw.io and Mermaid are planned. MIT licensed.

```
$ diagramdiff old.excalidraw new.excalidraw
old.excalidraw → new.excalidraw: 1 removed, 4 added
- arrow "Gateway" → "Queue"
+ rectangle "API"
+ rectangle #Aim8XXiCRUQBdeFuAj1Av
+ arrow "API" → #Aim8XXiCRUQBdeFuAj1Av : "HTTP"
+ arrow "API" → (unconnected)
```

## Install

You need Rust (https://rustup.rs).

```
cargo install diagramdiff --locked
```

## Usage

```
$ diagramdiff --help
Shows what changed between two Excalidraw files, in the diagram's own terms

Usage: diagramdiff [OPTIONS] <OLD> <NEW>

Arguments:
  <OLD>  The old diagram
  <NEW>  The new diagram

Options:
      --exit-code  Exit with 1 if there are Changes and 0 if not, like git diff --exit-code
  -h, --help       Print help
  -V, --version    Print version
```

This compares a diagram with its last commit, in bash or zsh.

```
diagramdiff <(git show HEAD:./docs/architecture.excalidraw) docs/architecture.excalidraw
```

## Reading the output

The first line names the two files and counts the changes. Each line after it is one change, `-` for removed and `+` for added. It names the kind, such as `rectangle` or `arrow`, and the label in quotes. An arrow also shows what it connects, from → to, and its own label after ` : `.

- `#Aim8XXiCRUQBdeFuAj1Av` has no label, so it shows its key, the Excalidraw element id.
- `(unconnected)` is an arrow end that isn't attached to anything.

diagramdiff matches by key only. A box you delete and draw again gets a new key, so it shows as removed and added, even with the same label. When a tool rebuilds the file with new keys, everything shows as removed and added.

This version shows only what was added and removed. A box you relabel, or an arrow you move to another box, keeps its key and shows nothing yet. Moving, resizing and restyling print nothing either. When nothing was added or removed, diagramdiff prints nothing.

## Exit code

diagramdiff exits 0 when the diagrams differ, like `git diff`, because a planned git diff mode needs it (git stops at a diff tool that exits with anything else); `--exit-code` makes it exit 1 on changes and 0 on none, like `git diff --exit-code`. Errors always exit 2.

## In code review

diagramdiff reads two files and prints the changes. Your CI shows that output to reviewers, and your platform's code owners decide who must approve a diagram change.

### Code owners

On GitHub, add a line to `.github/CODEOWNERS`. Then, in the branch protection rule or ruleset for your main branch, turn on "Require a pull request before merging" and its option "Require review from Code Owners". The team needs write access to the repository.

```
*.excalidraw @your-org/architects
```

On GitLab, add a line to `CODEOWNERS` in the repository root, `docs/` or `.gitlab/`, and turn on "Required approval from code owners" on the protected branch. Code owners is a GitLab Premium and Ultimate feature.

```
*.excalidraw @your-group/architects
```

### GitHub Actions

This workflow, saved as `.github/workflows/diagrams.yml`, prints the changes to every diagram in a pull request to the job log.

```yaml
name: Diagrams

on:
  pull_request:
    paths: ['**.excalidraw']

permissions:
  contents: read

jobs:
  diagramdiff:
    runs-on: ubuntu-latest
    steps:
      # Checks out the pull request merged into its base, so HEAD^1 is the base
      - uses: actions/checkout@v7
        with:
          fetch-depth: 2
      - run: cargo install diagramdiff --locked
      - shell: bash
        env:
          BASE: HEAD^1
        run: |
          git diff -z --name-only --diff-filter=M "$BASE" HEAD -- '*.excalidraw' |
          while IFS= read -r -d '' file; do
            git show "$BASE:$file" > "$file.old"
            diagramdiff -- "$file.old" "$file"
          done
```

### GitLab CI

This job in `.gitlab-ci.yml` does the same for a merge request.

```yaml
diagramdiff:
  image: rust:1
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
      changes: ['**/*.excalidraw']
  variables:
    GIT_DEPTH: "0"
  script:
    - cargo install diagramdiff --locked
    # The target branch in a merged results pipeline, else where the merge request branched off
    - BASE=${CI_MERGE_REQUEST_TARGET_BRANCH_SHA:-$CI_MERGE_REQUEST_DIFF_BASE_SHA}
    - |
      git diff -z --name-only --diff-filter=M "$BASE" HEAD -- '*.excalidraw' |
      while IFS= read -r -d '' file; do
        git show "$BASE:$file" > "$file.old"
        diagramdiff -- "$file.old" "$file"
      done
```

Both jobs diff only the diagrams changed in place. Added, deleted and renamed diagrams are skipped for now. If diagramdiff can't read a file, the job stops there and fails.
