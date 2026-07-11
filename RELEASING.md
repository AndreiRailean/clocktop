# Releasing New Versions

This project uses `cargo-release` to automate version bumping and changelog updates, and `cargo-dist` to build and upload binaries via GitHub Actions.

## Prerequisites

Ensure you have the required release tools installed locally:

```bash
cargo install cargo-release git-cliff
```

## Step-by-Step Release Process

### 1. Ensure your local branch is clean and up to date
Make sure you are on the `main` branch and have pulled the latest remote changes:
```bash
git checkout main
git pull origin main
```

### 2. Run a dry-run preview
Preview the version bump, git tags, and changelog generation to ensure everything parses perfectly without writing changes to disk:
```bash
# Options: patch (0.1.1), minor (0.2.0), major (1.0.0)
cargo release patch
```
*Review the terminal output to make sure the changelog formatting looks correct.*

### 3. Execute the release live
Run the execution command. This will bump the version in `Cargo.toml`, invoke `git-cliff` to update `CHANGELOG.md`, create a local git commit/tag, and push them to GitHub:
```bash
cargo release patch --execute
```

### 4. Verify the GitHub Build
Once pushed, the tag triggers the Release GitHub Action runner. 
1. Go to your repository on GitHub.
2. Click the **Actions** tab to watch the build complete.
3. Check the **Releases** page to verify the binaries were attached securely.
