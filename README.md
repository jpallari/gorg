# gorg

Git repository organiser. Features:

- Cloning and creating Git repos in a standard path structure.
- Fuzzy search for finding projects.
- Run command in each Git repo matching a fuzzy search.

## Installation

Prerequisites:

- [Rust toolchain](https://www.rust-lang.org/learn/get-started)
- Git CLI

After installing the prerequisites, you can compile a release binary from this repository:

```shell
git clone https://github.com/jpallari/gorg.git
cd gorg
cargo build --release
```

The binary will be placed to path `target/release/gorg` in the repository.
Install the binary to somewhere in your `PATH` to make the tool available in your shell.

## Usage

### Scan project directory for Git repositories

By default, gorg expects all projects to be found from directory `projects/` in your home directory.
You can change this in the configuration settings.

To make gorg aware of all individual projects, it will need to scan them and populate its internal index file.
You can do this with the following command:

```shell
gorg update-index
```

### Clone an existing project

You can clone an existing project using the following command:

```shell
gorg init https://github.com/jpallari/gorg.git
```

This will clone the given repository to a standard path structure based on the given Git URL:
`<projects directory>/github.com/jpallari/gorg`

Alternatively, you can specify the URL in a simplified way:

```shell
gorg init github.com jpallari gorg
```

This will automatically build the Git URL from the given parts.

### Initialise a new project

If you want to create a new project without cloning it, you can do with the following command:

```shell
gorg init --no-clone https://github.com/jpallari/gorg.git
```

This will set up a Git repository in the standard path based on the given Git URL (e.g. `<projects directory>/github.com/jpallari/gorg`), and set up a remote repository for the repo.

Alternatively, you can specify the URL in a simplified way:

```shell
gorg init --no-clone github.com jpallari gorg
```

This will automatically build the Git remote URL from the given parts.

### List projects

You can list all the projects in your project directory using the `list` sub-command:

```shell
gorg list
```

You can also search for the projects using a fuzzy query:

```shell
gorg list github
```

If you want to search for the projects using a prefix match instead, you can use the `-p` or `--prefix-search` flag:

```shell
gorg list -p github
```

If you want to display the full project path instead of just the project name, you can use the `-f` or `--full-path` flag:

```shell
gorg list -f github
```

### Find a project

You can use the `find` sub-command to activate an interactive fuzzy search for projects:

```shell
gorg find
```

When you type a query, matching projects will be listed.
You can select a project from the matches using up and down arrow keys or Ctrl+P and Ctrl+N key combinations.
Once you've selected a project, hitting the Enter key will print out the selected project and end the query.
You can cancel a selection using the Ctrl+C or Ctrl+D key combinations.

If you want to print out the full project path instead of just the project name on selection, you can use the `-f` or `--full-path` flag:

```shell
gorg find -f
```

All of the extra positional parameters will be used as part of the fuzzy query:

```shell
gorg find github
```

### Run a command in matching projects

You can run a command in all Git projects that match a query as follows:

```shell
gorg run --query github ls
```

The above command will make gorg enter each project matching query `github` and run `ls` there.

The flag `--query` (also available as `-q`) sets the fuzzy query, while the positional parameters are used as the command and the command arguments to run.

If you don't set the `--query` / `-q` flag, the command will be run on all projects.

If you are unsure which projects the command will be executed on, you can add the flag `-d` or `--dry` to just print out the project names.

```shell
gorg run --query github -d ls
```

### More information

For more details on all commands run `gorg --help` and `gorg <command> --help`.

## Configuration

Here's a full configuration file with default values.

```toml
# Configuration file for gorg
#
# This file is read from one of these locations
# depending on which environment variables are set:
#
# - Path set in environment variable GORG_CONFIG
# - Path $XDG_CONFIG_HOME/gorg/config.toml
# - Path ~/.config/gorg/config.toml
#

# Path where all of the Git repositories will be placed
projects_path = "~/projects"

# Path where the gorg index file will be stored
index_file_path = "~/projects/.gorg-db"

# Maximum number of items to list when finding projects interactively
max_find_items = 10

# Command to use for Git actions
git_command = "git"

# Name to use for the remote repository for new Git projects
git_remote_name = "origin"
```

## Tips

### Quickly jump to a project directory in your shell session

Add this shell command to your shell configuration file (e.g. `.bashrc`, `.zshrc`) to help jump between projects:

```shell
gcd() {
    local dir
    dir=$(gorg find -f "$@")
    if [ -n "${dir:-}" ]; then
        cd "$dir" || return 1
    else
        return 1
    fi
}
```

After that, when you run the command `gcd`, your shell will jump to the selected project directory in your shell session.

## License

[Apache License 2.0](LICENSE)

