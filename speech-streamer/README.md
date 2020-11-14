The idea of this module is to have a program that's _only_ responsible
for listening for audio input and outputting words.

- All output is on stdout
- If any control by the listening process is required, it will come on stdin
- ... jsonrpc api, to keep things dead simple (may look to use grpc in future but ecosystem still looks immature)

Voice recognition based on Mozilla's STT (based on commonvoice dataset)
