# Motivation

Rust programs are known to be monolithic, bringing in foreign crates even if an equivalent library is already present on the system. With the good will of keeping programs from breaking changes in system upgrades, and making docker-less deployment an easy feat.

However, the difficulty of communicating with external resources prevents meaningful extensions to be existing programs. Without recompilation, user-defined customisation is typically achieved through one of the followings:
- Embedded scripting engine
- And in rare cases, soft-linked .so files

In both cases, the implementation for extensions are both language specific and the running of those extensions are fully dependent on the parent program, limiting their ability to utilise system resources.

This does not have to be the case.

*ccanvas* attempt to solve this problem for TUI programs - instead of programming all features into a single binary, a blank canvas is opened for external programs to draw on, where they communicate through messages.

A proof-of-concept has been created successfully at *ccanvas_old*. This rewrite hopes for a more optimised implementation with a refined protocol for real world use.

> "Any sufficiently sophisticated TUI program will eventually turn into a text editor."
> 
> ~ Some guy from Virbox's community.

Or maybe a window manager.
