# ghost-text.hx

A Helix plugin written in Rust, that lets you use Helix to edit text inputs in the browser!

## Preview

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
git checkout 2fe135cf
```

Install Steel and the `forge` package manager:

```sh
cargo xtask steel
```

Install the plugin itself:

```sh
forge pkg install --git https://github.com/nik-rev/ghost-text.hx.git
```
