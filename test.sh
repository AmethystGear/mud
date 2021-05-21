#!/bin/bash

pkill python
(cd mirae_client && nohup python client.py 8000 9000 127.0.0.1 > client_log &)
(cd mirae_server && cargo run --release 9000 $1 $2)

