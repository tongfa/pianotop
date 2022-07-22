#!/bin/bash

set -e

PP_IFACE=$(amidi -l | grep 'Virtual Raw' | head -n1 | awk '{print $2}')

if [ "" == "$PP_IFACE" ] ; then
  echo "please run: sudo modprobe snd_virmidi midi_devs=1"
  exit 1
fi
