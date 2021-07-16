while true; do
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
        ./run.sh seed $RANDOM
    fi
    sleep 10s
done
