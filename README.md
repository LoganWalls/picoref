# PicoRef
A simple and minimal CLI tool to manage your references.
PicoRef is based on plain text: your library is just a folder, and each reference is a subfolder within that library. The metadata for each reference is stored in the [CSL JSON](https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html) format, but is encoded as `yaml` to facilitate manual edits. 

PicoRef's scope of functionality is deliberately small. It focuses on just a few things:
1. Managing the files that make up your library (e.g. creating directories when needed, placing PDFs in the same subdirectory as the reference metadata, tags for managing collections)
2. Adding new references to your library from a DOI using the CrossRef API and various APIs provided by preprint servers
3. Fetching open access PDFs when available via preprint servers or UnPaywall

For more advanced functionality, I encourage you to compose PicoRef with other CLI tools like [`ripgrep`](https://github.com/BurntSushi/ripgrep), [`fzf`](https://github.com/junegunn/fzf), [`just`](https://github.com/casey/just), and [`watchexec`](https://github.com/watchexec/watchexec). Otherwise, if you are looking for additional features there are also excellent alternatives like [Papis](https://github.com/papis/papis), and [Zotero](https://www.zotero.org/).

## Getting started
1. Make sure you have installed the [rust toolchain](https://www.rust-lang.org/tools/install)
2. Clone this repository and `cd` into it
3. Compile and install the binary: `cargo install --path .`
4. Create a configuration file (see below)

## Configuration
PicoRef will search for a configuration file in the following locations (in order of priority):
1. `$XDG_CONFIG_HOME/picoref/config.toml`
2. `$HOME/.config/picoref/config.toml`

The configuration file should contain two values:
```toml
root = "/absolute/path/to/your/library/folder"
email = "your@email.com"
```
(Email is used to access the polite pool of some APIs when fetching metadata or PDF files)

## How do I...
### Add a new reference
```sh
picoref add "<DOI>"
```

### Download an Open Access PDF for a reference
```sh
picoref pdf "<citekey>"
```

### Export references for use with other tools like LaTeX or Pandoc (via citeproc)
```sh
picoref export "path/to/export/json"
```

### Keep track of different collections / groups of references
Use tags (see below)

### Export on the references associated with a specific project
1. Choose a tag name for the project (e.g. `my-project`)
2. Apply that tag to all of the references associated with the project
3. Filter your export using the tag:

```sh
picoref export "path/to/export/json" --tag 'my-project'
```

### Add a tag to a reference
```sh
picoref tag add "<tag>" "<citekey>"
```

### Find all references with a given tag
```sh
picoref list --tag "<tag>"
```

### View a list of all unique tags in your library
```sh
picoref tag list
```

### Migrate from Zotero
Note: it is not yet possible to migrate attachments. The instructions below will only migrate your references themselves.

1. Install the [BetterBibTeX](https://retorque.re/zotero-better-bibtex/) plugin for Zotero
2. Open Zotero
3. Right-click on `My Library`
4. Click on `Export Library...`
5. Choose the format: `Better CSL JSON`
6. Click OK
7. Choose a path to export your JSON file

Then:
```sh
picoref import "path/to/your/file.json"
```

### Delete a reference
```sh
rm -r "$YOUR_LIBRARY/<citekey>"
```

## Why make this?
- I wanted a very simple way to manage my references in plain text
- [Papis](https://github.com/papis/papis) looks great, but installing python apps with dependencies is a pain and I only need a small subset of its functionality
- I wanted to practice using Rust, and this seemed like a good project with a reasonably small scope
