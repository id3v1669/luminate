# Rust UIKit for Iced

A flexible, native, runtime-switchable UI kit for [`iced`](https://github.com/iced-rs/iced) that allows you to define buttons, inputs, and other widgets in a reusable way.

Unlike typical iced widgets, this library allows you to:

- Directly emit application messages (`AppMessage`) from UIKit widgets.
- Switch between implemented themes at runtime without incurring significant overhead.
- Cache elements for zero-overhead rendering in hot paths.
- Clean API for building UIs declaratively.

---

## Features

- **Generic `Kit`**: UIKit can be generic over your application `Message`.
- **Runtime theme switching**: Choose different themes at runtime using a simple strategy.
- **Message passthrough**: Send application-specific messages directly from UIKit widgets.
- **Extensible**: Easily implement new themes without changing your application logic.

---

## Project State

Todo list tracks implemented components and features from the [web version of AuraVibe UIKit.](https://github.com/sonata-ltd/launcher/tree/master/app/src/uikit/components)

### Available Components

- [x] Button
- [ ] Card
- [ ] CodeComponent
- [ ] Dropdown
- [ ] Grid
- [ ] Indication
- [x] Input
    - [x] Label
    - [x] Hint
    - [x] Tooltip
- [ ] Progress
- [ ] Section
- [ ] Separator
- [ ] Sidebar
- [ ] Spinner
- [ ] Window

### Available Features

- [ ] Animations
    - [x] Physically correct spring implementation

---

## Example

1. Reserve UI Kit cell in application state

```rust
struct Data {
    input_content: String::new(),
    uikit: Box<dyn for<'a> Kit<'a, Message>>,
}
```

2. Write a `new()` implementation:

```rust
impl Data {
    fn new<K>(kit: K) -> (Self, Task<Message>)
    where
        K: for<'a> Kit<'a, Message> + 'static,
    {
        (
            Self {
                uikit: Box::new(kit),
                input_content: String::new(),
            },
            Task::none(),
        )
    }

    // update(), view() functions...
}
```

3. Write a mapper implementation inside `Data` for clean building API:

```rust
impl Data {
    // new() function...

    fn kit_mapper(&self) -> UIMapper<'_, Message> {
        UIMapper::new(&self.uikit)
    }

    // update(), view() functions...
}
```

4. Pass chosen default UI Kit on application start:

```rust
fn main() -> iced::Result {
    iced::application(move || Data::new(Sonata::new()), Data::update, Data::view)
        .run()
}
```

5. Build interface using simplest `button` widget:

```rust
impl Data {
    // new(), update(), kit_mapper() functions...

    fn view(&self) -> Element<'_, Message> {
        let kit = self.kit_mapper();

        kit.button().label("Action").on_press(Message::Pressed).into()
    }
}
```

---

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
iced_auravibe = { git = "https://github.com/sonata-ltd/auravibe", branch = "master" }
```
