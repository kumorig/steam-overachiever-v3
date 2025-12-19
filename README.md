# Steam Overachiever v3

A desktop application for tracking your Steam game library and achievement progress over time. 

This project is in no way affiliated with or endorsed by Valve Corporation.


## Setup

For it to work, you need to provide your Steam API key in a `.env` file in the project root. You can get one from [here](https://steamcommunity.com/dev/apikey) after registering a developer account (free for now).

See `.env.example`. You should be able to just rename it to `.env` and fill in your API key. But if you build an exutable, the `.env` file should be placed alongside the executable.


## Building
Make sure you have [Rust](https://rust-lang.org/tools/install/) installed. Then, run:

```bash
cargo run --release
```
or 
```bash
cargo build --release
```

## Contributing
Contributions are welcome. Make a PR or open an issue. 
About half of the code has been "vibe-coded", feel free to help clean-up the mess. Thus, AI contributions are welcome, but at least do some low effort testing before submitting a PR. Thanks!

## Roadmap
None really, but feel free to open an issue with suggestions.
I plan to improve the graphs, but I'll run the app a few weeks so I have some data some data to work with first.

## License
This project is licensed under the MIT License. See the `LICENSE` file for details.

## Acknowledgements
This project is in no way affiliated with or endorsed by Valve Corporation.