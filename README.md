# qsh

qsh is a shell project focused on **easy configuration**, **extensibility**, and a clean user experience.

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

## Installation

```bash
git clone https://github.com/KoTTana24/qsh
cd qsh
cargo build --release
```

## Development status

qsh is in an early development stage.

Current priorities:

* [ ] Plugin architecture
* [ ] qpm implementation
* [ ] Documentation
* [ ] First stable release

## Prompt widgets and colors

Prompt placeholders can be colored independently. Custom prompt widgets can be
provided as strings and use the same color map:

```lua
theme = {
    prompt = "{current_directory}@{username} {time} {project} > ",
    prompt_colors = {
        current_directory = "#5fafaf",
        username = "#d787ff",
        project = "#87d75f",
        time = "#808080",
    },
    widgets = {
        project = "qsh",
    },
}
```

Built-in placeholders are `{username}`, `{current_directory}`, and `{time}`.
The time widget displays local time in `HH:MM:SS` format. Any key in
`theme.widgets` can be used as a custom placeholder, and its color is selected
by the matching key in `theme.prompt_colors`.

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
