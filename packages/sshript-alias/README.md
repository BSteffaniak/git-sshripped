# git_sshript

Alias binary for git-sshripped, installed as `git-sshript`.

## Overview

This crate is a thin wrapper that delegates entirely to
`git_sshripped_cli::run()`. It exists solely to provide the shorter
`git-sshript` command name as an alternative to `git-sshripped`.

## Usage

```sh
git sshript unlock
git sshript status
```

See the [git-sshripped repository](https://github.com/BSteffaniak/git-sshripped)
for full documentation.
