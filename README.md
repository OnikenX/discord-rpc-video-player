# Discord RPC Video Players

This one is a little service that detects processes with the terms vlc or mpv and tries to get the window name of that process, it updates the process list in each 10 secs.

It depends on:

- ps
- xdotool

And for building you need the [discord sdk](https://discord.com/developers/docs/game-sdk/sdk-starter-guide) with the variable **DISCORD_GAME_SDK_PATH** holding the path for it.

It only works on linux with x11.