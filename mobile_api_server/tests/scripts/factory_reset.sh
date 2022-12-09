#!/bin/bash

# Unit test is checking that script was run using D-Bus
SCRIPT="${0##*/}"
busctl --user call eu.sifis_home.Testing.FactoryReset /Testing eu.sifis_home.Testing ScriptWasRun s "${SCRIPT}"
