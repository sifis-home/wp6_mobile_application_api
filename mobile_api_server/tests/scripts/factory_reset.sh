#!/bin/bash

# Unit test is checking that script was run using D-Bus
SCRIPT="${0##*/}"
dbus-send --type=method_call --dest=eu.sifis_home.Testing.FactoryReset /Testing eu.sifis_home.Testing.ScriptWasRun "string:${SCRIPT}"
echo "${SCRIPT} was run"
