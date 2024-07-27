#!/bin/bash

# ==================================
# 負荷試験スクリプト。
# ==================================


if [[ $HOSTNAME == stress-* ]];
then
    TEAM_ID=$1
	CLIENT_ORIGIN_URL="https://$TEAM_ID.ftt2407.dabaas.net"
else
	CLIENT_ORIGIN_URL="http://nginx"
fi

if [[ -n "$2" ]];
then
    FILE_NAME=$2
else
    FILE_NAME=`date "+%Y%m%d_%H%M%S"`
fi

LOG_FILE_PATH="./logs/${FILE_NAME}.json"
RAW_DATA_FILE_PATH="./scores/raw-data-${FILE_NAME}.json"
SCORE_FILE_PATH="./scores/score-${FILE_NAME}.json"

# 負荷試験開始
echo "負荷試験を開始します。"

if [[ $HOSTNAME == stress-* ]];
then
    k6 run --out json=${LOG_FILE_PATH} main.js -e CLIENT_ORIGIN_URL=${CLIENT_ORIGIN_URL} -e RAW_DATA_FILE_PATH=${RAW_DATA_FILE_PATH} && \
    bash ./calculate_score.sh $LOG_FILE_PATH $SCORE_FILE_PATH $RAW_DATA_FILE_PATH $TEAM_ID
else
    docker run --name k6 --rm --network webapp-network \
      -v $(pwd):/usr/src/benchmarker \
      -v /usr/src/benchmarker/node_modules \
      -it 42tokyo2407.azurecr.io/benchmarker:latest \
      /bin/bash -c "k6 run --log-output=file=${LOG_FILE_PATH},level=warning --log-format json main.js -e CLIENT_ORIGIN_URL=${CLIENT_ORIGIN_URL} -e RAW_DATA_FILE_PATH=${RAW_DATA_FILE_PATH} && bash ./calculate_score.sh $LOG_FILE_PATH $SCORE_FILE_PATH $RAW_DATA_FILE_PATH"
fi
