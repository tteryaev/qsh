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

## Key bindings

`break_line` cancels the current input and starts a fresh prompt. `clear_line`
empties the current input. Both actions can be rebound in the top-level
`binds` table:

```lua
binds = {
    break_line = "ctrl+c",
    clear_line = "ctrl+u",
    exit = "ctrl+d",
    clear_screen = "ctrl+l",
    move_start = "ctrl+a",
    move_end = "ctrl+e",
    delete_word_backward = "ctrl+w",
    delete_to_end = "ctrl+k",
    history_search = "ctrl+r",
}
```

`exit` leaves qsh immediately, and `clear_screen` clears the terminal while
keeping the current input. `move_start` and `move_end` move to the start or end
of the input. `delete_word_backward` removes the previous word, and
`delete_to_end` removes text from the cursor to the end. Bindings accept key
names such as `esc`, `enter`,
`tab`, `up`, `left`, and
modifier combinations such as `ctrl+c` or `alt+left`. The default `clear_line`
binding is Ctrl+U; set it to `ctrl+c` for fish-style Ctrl+C behavior.
`history_search` searches by the command name at the start of each history
entry and lets you cycle through matching commands.

Custom bindings use the key combination as the table key and a Lua function as
the value. The function receives a `line` table; update `text` and `cursor` to
change the input:

```lua
binds["ctrl+alt+u"] = function(line)
    line.text = string.upper(line.text)
    line.cursor = utf8.len(line.text)
end
```

The cursor position is a character index, not a UTF-8 byte offset.

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
