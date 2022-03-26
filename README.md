# Discord RPC Video Players

This one is a little service that detects processes with the terms vlc or mpv and tries to get the window name of that process, it updates the process list in each 10 secs.

It depends on:
- ps
- xdotool

It only works on linux with x11.