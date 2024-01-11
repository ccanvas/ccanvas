# ccanvas

ccanvas allows multiple programs to draw on the same terminal in an async/non-blocking manner. To create a ccanvas component, see [libccanvas](https://github.com/Siriusmart/libccanvas).

> **ccanvas uses unix sockets**, which may not be available on windows.

## Showcase

Click on the image for video.

[![](https://gmtex.siri.sh/api/usercontent/v1/file/id/1/tex/Dump/Showcases/ccanvas-snake.png)](https://gmtex.siri.sh/fs/1/Dump/Showcases/ccanvas-snake.webm)

[Source](https://github.com/Siriusmart/libccanvas/tree/master/examples/snake)

## Backstory

The [**`youtube-tui`**](https://github.com/Siriusmart/youtube-tui/) was my first TUI project, one major issues is that the whole program freezes when waiting for server response of a video. This is because it uses a **blocking event loop** as so:

1. Render current state to screen.
2. Do something - such as loading a video.
3. Repeat.

This is problematic if the task takes a long time to complete, as it blocks all incoming events, including pressing `q` which exit the TUI.

At the same time I was also facing the unsolvable problem of inter-component communication, I hacked together a solution which utilises shared memory, but it was far from the ideal message-based communication.

## Goals

- To create a TUI framework that is non-blocking by default.
- To create a component based TUI framework, where each component is a separate program.
- To create a component based TUI framework, where each component have the option to communicate with each other using messages, shared memory, or shared storage.

## Implementation

### Components

ccanvas is structured into "components"

- a ***space*** can contain any number of child components.
- a ***process*** represents a real process running.

> A space can focus on itself, or one child space.

Each component is referenced by its unique identifying ***discriminator***, which is just a path of numbers, such as `/1/2/3/4`. `/1` is called the ***master space***.

- New spaces can be created and new processes can be spawned in.
- Any component can be ***dropped*** and removed.
- To exit the canvas, call ***drop*** on `/1.`

### Events

ccanvas receive "events" from 3 sources.

- Terminal events - key presses, mouse click, resizes, etc.
- Component requests - messages to be passed, render requests, etc.
- Generated events - registering subscriptions, message broadcasts, etc.

All 3 of these event sources are funnelled into a single ***event stream***, where they are handled by a non-blocking event loop, here's how it works:

1. Events enters the canvas through different sources.
2. All events are funnelled into a single ***event stream***.
3. All events are passed into the ***master space***.

The space will then decide where should the event be passed to. First it will pass to all the processes subscribed to the event, and then it will pass to the ***focused space*** (if there is one).

## TODO

One more feature needed before I work on a window manager component - list components in a space.
