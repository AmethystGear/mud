#!/bin/bash
(cd mirae_client && nohup python client.py 31415 9000 > client_log &)
(cd mirae_server && cargo run --release 9000 $1 $2)

