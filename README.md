# hue-cli

hue-cli allows you to control your Philips Hue lights via the command prompt.

# Why?

This was created as a simple project to get used to the Rust programming language.

## Build Instructions

This can be built using the Rust package manager Cargo

```console
cargo build
```

## Usage

To use this tool it requires a Philips Hue Bridge

Login to the Hue Bridge
```console
hue-cli login
```

List connected lights along with status 
```console
hue-cli list
```

Turn on a light 
```console
hue-cli set on "Living Room Light"
```

Turn off a light 
```console
hue-cli set off "Living Room Light"
```

Set brightness of a light
```console
hue-cli set bri "Living Room Light" 50
```

## Features
- [x] Login to Hue Bridge
- [x] Turn on light
- [x] Turn off light
- [x] Set brightness of light
- [ ] Set colour of light
- [ ] Set Alias Names for lights to easily reference them later

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License
[MIT](https://github.com/JamesStallworthy/hue-cli/blob/master/Licence.md)
