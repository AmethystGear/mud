(nohup ./run.sh seed $RANDOM > mirae_server/server_log &)
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
	(nohup ./run.sh seed $RANDOM > mirae_server/server_log &)
    fi
    sleep 10s
done
