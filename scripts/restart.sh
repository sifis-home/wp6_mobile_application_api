#!/bin/bash

# We can make the system reboot with one of the following commands:
#
# systemctl reboot
# reboot
# shutdown -r now

# However, we only print a message to indicate that the script was run
SCRIPT="${0##*/}"
echo "${SCRIPT} was run"
