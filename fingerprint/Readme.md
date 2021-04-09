Library for context-based authentication.
Build with "cargo build" and test with "cargo test".  There's an external dependency on ALSA, which should be installed separately; all other dependencies are managed by cargo.
Main function receives three arguments, passed via command-line:
    mode: "r" to await a pairing or message request, "p" to send a pairing attempt or "s" to send a message request.  Note that send and pair will fail if there's nothing receiving.  
    addr: the address to listen at if receiving or to send to if sending.
    dev_name: the name of the recording device.

    Send and pair will return a vec of non-zero length on a success, a vec of zero length if everything sent and received but no successful decoding occurred, and an error if a network issue occurred.  Receive will return the message it sent on a successful communication, and an error on a failed communication; it does not know whether the decoding succeeded or failed on the other end.
    Note - performance in debug mode is pretty poor.  Prefer to use release mode 
Other parameters of interest are defined in the config object created in main, and can be adjusted as desired.