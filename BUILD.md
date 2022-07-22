# Required dependencies


```
apt install build-essential libasound2-dev
```

# development

## if you do not have an ALSA device connected

First, you need some kind of ALSA device to connect to.  In case you don't have one
this will setup a virtual interface:

```
to setup 2 virtual interfaces:
`sudo modprobe snd_virmidi midi_devs=1`
```

you probably don't want to use Midi-through-0 btw...

There are scripts in ./test/ to send notes to a virtual ALSA device

## running the parts

```
(cd recorder ; cargo run)
```

then in another shell:

```
(cd ui ; yarn serve)
```
