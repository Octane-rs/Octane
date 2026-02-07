# Octane

High-performance, real-time Android screen mirroring and control.

## Disclaimer

This project is in beta and will undergo heavy changes.

## Architecture

Implementation of the [ELM architecture](https://redbadger.github.io/crux/guide/elm_architecture.html) combined
with [rust actors](https://ryhl.io/blog/actors-with-tokio).

## Motivations

The current open source client for scrcpy is suffering performance limitations especially video stuttering at 120fps,
and the non-reliable keybinding layer.
This project aims to fix this issues while the final goal is to implement direct Vulkan rendering.

In this first implementation video decoding is arleady proven stable while leveraging hw decoding.

## Findings

On compatible devices h265 is a net improvement over h264. Allowing to use half the bitrate for the same visual result,
it is as well consuming 1/3 to half of the decoding resources using the vulkan decoder. The downside is the heavy cost
of encoding for the Android device that will quickly overheat supposing other tasks are concurrently running.

The AV1 codec is way too heavy to be usable.

## HW decoding

So far vulkan hw decoding is implemented using ffmpeg decoder. The issue is the lack of communication between the
rendering (wgpu) vulkan device and the ffmpeg one. The goal here is to implement direct rendering, meaning the frame is
not copied to the cpu for the pixel conversion, but stays in VRAM.

## TODO

First time not using a monorepo and it is plain awful, let's refactor that.

## Utils

```bash
Get-ChildItem -Path src -Filter *.rs -Recurse | ForEach-Object { ( $_.FullName) }
```
