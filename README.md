# ghost-text.hx

A Helix plugin written in Rust, that lets you use Helix to edit text inputs *in the browser*!

## Preview

https://github.com/user-attachments/assets/5b79390f-31f2-4e1e-adab-ce01ad39e355

## Usage

1. To start: `:ghost-text-start` 
1. Enable the Ghost Text browser extension, and select a text input
1. The active buffer and the selected text input will be synced!

To stop: `:ghost-text-stop`

## Installation

Right now, Helix plugins are available only as a pull request, which you will have to build to use.

Build Helix with the plugin system:

```sh
git clone git@github.com:mattwparas/helix.git helix-plugin
cd helix-plugin
git checkout 081f9e86ed63fe738c609e0fc944db10c150d4fd
```

Install Steel and the `forge` package manager:

```sh
cargo xtask steel
```

Install the plugin itself:

```sh
forge pkg install --git https://github.com/nik-rev/ghost-text.hx.git
```

<!-- You will also need to install the dylib manually ([this process will be simpler in the future](https://github.com/mattwparas/steel/pull/424)): -->

<!-- ```sh -->
<!-- cd ~/.steel/cog-sources/ghost-text.hx -->
<!-- cargo steel-lib -->
<!-- ``` -->

Add it in your `~/.config/helix/init.scm`:

```sh
(require "ghost-text/ghost-text.scm")
```

The above will add the `:ghost-text-start` and `:ghost-text-stop` commands automatically.
