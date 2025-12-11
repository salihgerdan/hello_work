# Hello Work <img src="img/hello_work_pixel.png" width="40"/>

It's cute, it's a pomodoro timer, it's a project manager? to-do list (of sorts), and it's a time tracker.

![Main Window](img/screenshots/screenshot_main.png?raw=true)

You can add projects to track time, and an arbitrary depth of sub-projects which you might also call tasks, there's no limit.

There's a mini-window to always stay on top of your screen to remind you that a session is going.

![Mini Window](img/screenshots/screenshot_mini.png?raw=true)

You can see your total hours for the last week (and soon™ other stats too).

![Mini Window](img/screenshots/screenshot_stats.png?raw=true)

## FAQ

### The UI elements are too large under X11

Either set `Xft.dpi: 96` in `~/.Xresources`, or set `WINIT_X11_SCALE_FACTOR=1`. See [this](https://github.com/rust-windowing/winit/issues/2231).
