Audio Bicycle
=============
A two-audio communicator that uses [the VBAN protocol](https://vb-audio.com/Voicemeeter/VBANProtocol_Specifications.pdf) to pass audio to another computer.

## Usage
This is intended to work on any machine with PulseAudio, but has only been tested on Linux.

For Linux, add a `~/.config/audio-bicycle/config.toml` file with the following contents:
```toml
local_address = "<local IP address>:<local port>"
dest_address = "<destination IP addres>:<destination port>"
stream_name = "<name>"
```
Replace each item in `<angle brackets>` with the appropriate value. By default, VBAN uses port 6980, so if you're unsure
what to use, try that.

Then, `cargo install audio-bicycle` and run `audio-bicycle`. It's probably best to set it up as a service:
```systemd
[Unit]
Description=audio-bicycle
BindsTo=pipewire-pulse.service # or pulseaudio.service
After=pipewire-pulse.service # or pulseaudio.service

[Service]
Type=simple
ExecStart=/home/<your user>/.cargo/bin/audio-bicycle # or wherever you installed it
Restart=on-failure

[Install]
WantedBy=default.target
```
