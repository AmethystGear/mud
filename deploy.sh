#!/bin/bash
pkill python
pkill mirae
sleep 10s
R=$(shuf -i0-1000 -n1)
(./run.sh seed $R &)
while true; do
    git remote update
    git checkout master
    UPSTREAM=${1:-'@{u}'}
    LOCAL=$(git rev-parse @)
    REMOTE=$(git rev-parse "$UPSTREAM")
    BASE=$(git merge-base @ "$UPSTREAM")
    if [ $LOCAL = $BASE ] && [ $LOCAL != $REMOTE ]; then
        git pull origin master
        pkill python
        pkill mirae
	    sleep 10s
        R=$(shuf -i0-1000 -n1)
	    (./run.sh seed $R &)
    fi
    sleep 10s
done
