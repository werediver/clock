target extended-remote /dev/cu.usbmodem98B6AFC21

monitor swdp_scan
attach 1

# Make sure "--se <program>" is supplied to GDB.
load

# Ensure the program will run after GDB quits.
b main
run
detach
