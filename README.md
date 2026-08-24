# qsh

A lightweight and configurable shell written with simplicity in mind.

qsh is a shell project focused on **easy configuration**, **extensibility**, and a clean user experience.

### The project is currently in early development. Many planned features are not implemented yet, but qsh is actively evolving.

## Features

### Currently

* Basic shell functionality
* Command execution
* Simple configuration system

### Planned

* Easy-to-use configuration
* Built-in plugin manager
* Plugin support through **qpm (qsh Plugin Manager)**
* Customizable shell environment
* Developer-friendly API for extensions

## qpm — qsh Plugin Manager

One of the main goals of qsh is a built-in plugin system.

**qpm (qsh Plugin Manager)** will allow users to:

* Install and manage qsh plugins
* Extend shell functionality
* Share custom commands and features
* Configure qsh without modifying the source code

Example of future usage:

```bash
qpm install plugin-name
qpm remove plugin-name
qpm list
```

*(qpm is currently planned and not available yet.)*

## Configuration

qsh is designed around a simple configuration approach.

Configuration example:

`file ~/.config/qsh/config.lua`

```lua
theme = {
	promt = "[{username}] {current_directory} -> ",
}

aliases = {
	c = "clear",
	n = "nvim",
    ll = "ls -la"
}
```

## Installation

Currently, qsh is built from source.

```bash
git clone https://github.com/KoTTana24/qsh
cd qsh
cargo build --release
```

The installation process may change in future releases.

## Development status

qsh is in an early development stage.

Current priorities:

* [ ] Stable command execution
* [ ] Configuration system
* [ ] Plugin architecture
* [ ] qpm implementation
* [ ] Documentation
* [ ] First stable release

## Contributing

Contributions, ideas, and feedback are welcome!

If you have suggestions or want to help with development, feel free to open an issue or submit a pull request.

---

## About the project

qsh is created as an experiment to build a modern shell focused on:

* simplicity instead of complexity
* customization without complicated configuration
* extensibility through plugins
* comfortable everyday usage

The project is inspired by existing shells but aims to create its own approach to shell design.

---
