Port of my AST3 asteroids clone Bevy/rust from Unity/C#.

This is my first rust program. Be gentle.

Still working on it, but actually playable.

You can play it here: [AST4 Game Link[https://jmonasterio.github.io/ast4/]



NOTES to SELF:

# TO BUILD FOR WASM/WEBG: [instructions][https://dev.to/sbelzile/making-games-in-rust-deploying-a-bevy-app-to-the-web-1ahn]
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./target/web --target web ./target/wasm32-unknown-unknown/release/ast4.wasm
npx serve

Link to OLD unity version with playable link: https://github.com/jmonasterio/ast3

- Followed these rust setup instructions: https://stackoverflow.com/questions/46885292/how-to-launch-a-rust-application-from-visual-studio-code#:~:text=Using%20Tasks%20Shortcut%20to%20run%20the%20Task%3A%20Ctrl,the%20project%2C%20change%20the%20contents%20of.vscode%2Ftasks.json%20as%20follows%3A

OLD: Using nighly because I want fast compile which requires:
- Fast compile / "dyanmic" makes LLDB fail in bevy. Sortof a hint here: https://stackoverflow.com/questions/67036895/dll-lookup-fails-on-application-load-time