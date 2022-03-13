Port of my AST3 asteroids clone to Bevy/Rust from Unity/C#.

You can play it here: [AST4 Game Link](https://jmonasterio.github.io/ast4/)


This is my first Rust program. Be gentle.

Still working on it, but actually playable.


Link to *old* unity version [old version](https://github.com/jmonasterio/ast3)


NOTES to SELF:

- Notes on how to build for webgl: [instructions][https://dev.to/sbelzile/making-games-in-rust-deploying-a-bevy-app-to-the-web-1ahn]


- Followed these rust setup instructions: https://stackoverflow.com/questions/46885292/how-to-launch-a-rust-application-from-visual-studio-code#:~:text=Using%20Tasks%20Shortcut%20to%20run%20the%20Task%3A%20Ctrl,the%20project%2C%20change%20the%20contents%20of.vscode%2Ftasks.json%20as%20follows%3A

- Nighly allows fast-compile, but "dyanmic" makes LLDB fail in bevy. Sortof a hint here: https://stackoverflow.com/questions/67036895/dll-lookup-fails-on-application-load-time