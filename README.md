# Reimplement the classical Snake game in Rust

## Build

```shell
cargo build
```

## Run

```shell
cargo run
```

## Feedback

- All kinds of feedback are welcome!
- Feel free to give me any advice on better design patterns I should use with Rust.

## Game structure

The game is organized into two main modules: `snake_game_scene` and `game_engine`.

The `snake_game_scene` module controls the game logic; it receives events when to draw the hud, when the objects were updated, etc. This module receives these events because it has a struct that implements the trait `GameScene` available on the `game_engine` module.

The `game_engine` runs any scene that was created for it. It allows some objects to be added to the struct `Game` in `game_engine` and make calls to update them and let them draw themselves when it's time. Any `struct` that implements the trait `GameObject` could be added to the `Game` struct.

The game also has modules for the objects added to the game. So the modules `snake` and `apple` have structs that implement the trait `GameObject`.
