# Prefab converter

This application converts prefabs from [Discohook](https://discohook.org).

## What does this tool do?
It converts base64 to toml, for prefabs.
It is used to store Discohook's output in an easier to use markup language.

## How do I use this?
1. In your prefabs folder, create a new file, which either contains base64 (from
  the url) or the JSON that the base64 represents.
2. Run the tool:
```bash
$ cargo r --bin prefab-converter -- file.json
```
3. Your file is now converted to toml. (with a .toml extension, of course)

## Why use this?
To make the format that has been generated easier readable, and easier to
quickly edit.
