Library for context-based authentication.
Build with "cargo build" and test with "cargo test".  There's an external dependency on ALSA, which should be installed separately; all other dependencies are managed by cargo.
Main function receives three arguments, passed via command-line:
    mode: either "r" to receive or "s" to send.  Note that send will fail if there's nothing receiving.
    addr: the address to listen at if receiving or to send to if sending.
    dev_name: the name of the recording device.
Other parameters of interest are defined in the config object created in main, and can be adjusted as desired.