# Hello Work <img src="img/hello_work_pixel.png" width="40"/>

It's a pomodoro timer, project manager, to-do list. A time tracker? Yeah, if you can quantize all your work into pomodoro sessions.

<img src="img/screenshots/screenshot_main.png" width="300"/>

You can add projects and an arbitrary depth of sub-projects. The total hours spent on each project will be displayed in the projects tab, aggregated with its sub-projects.

There's a mini-window to always stay on top of your screen to remind you that a session is going.

![Mini Window](img/screenshots/screenshot_mini.png?raw=true)

You can see your total hours for the last week (and soon™ other stats too).

<img src="img/screenshots/screenshot_stats.png" width="300"/>

## FAQ

### Does it save my session when I click "Stop"?

Not currently, no. This is to encourage finishing what you started. It does however, undermine the time tracking promise a bit.

### Mac tells me this is trash

<img src="img/screenshots/mac_trash.jpeg" width="300"/>

As this is an unsigned app, you will have to remove the quarantine after installation with the following command in the terminal.

```sh
xattr -d com.apple.quarantine /Applications/HelloWork.app
```

### The mini window does not stay on top under Wayland

Yeah that might happen due to unresolved Wayland limitations, use a window rule from your desktop.

### The UI elements are too large under X11

Either set `Xft.dpi: 96` in `~/.Xresources`, or set `WINIT_X11_SCALE_FACTOR=1`. See [this](https://github.com/rust-windowing/winit/issues/2231).

# Credits

[MonkeyType](https://github.com/monkeytypegame/monkeytype/tree/master/frontend/static/themes) for an excellent source of simple color schemes.
