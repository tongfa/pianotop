IFACE=hw:1,0

SELF_DIR="$( dirname -- "${BASH_SOURCE[0]}"; )";
. ${SELF_DIR}/play-note.inc

ON=0.40
OFF=0.10
VEL=4F

play_note 3C
play_note 3C
play_note 3E
play_note 40

play_note 3C
play_note 40
ON=0.90
play_note 3E
