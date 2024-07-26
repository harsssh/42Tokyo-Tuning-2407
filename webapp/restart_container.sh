#!/bin/bash

# ==================================
# アプリのコンテナ再起動スクリプト。
# ==================================

export DOCKER_BUILDKIT=1

# アプリのコンテナ再起動
echo "アプリのコンテナの再起動を開始します。"

# ネットワークの存在確認と作成
NETWORK_NAME="webapp-network"
if ! docker network ls | grep -q "$NETWORK_NAME"; then
    echo "ネットワーク $NETWORK_NAME が存在しないため新たに作成します"
    docker network create $NETWORK_NAME
fi

if [[ $HOSTNAME == app-* ]]; then
    docker compose \
        -f docker-compose.yml \
        -f pprotein.compose.yaml \
        -f grafana.compose.yaml \
        down --volumes --rmi local
	HOSTNAME=$HOSTNAME docker compose \
        -f docker-compose.yml \
        -f pprotein.compose.yaml \
        -f grafana.compose.yaml \
        up --build -d
else
    echo "ローカル環境でのコンテナ再起動を開始します。"
    # init.sh実行時には実行しない
    if [ -f ./../.da/.initLock ]; then
        docker compose down db --volumes --rmi local
    fi
	docker compose -f docker-compose.local.yml up --build -d
fi

if [ $? -ne 0 ]; then
    echo "コンテナの再起動に失敗しました。"
    exit 1
else
    echo "コンテナの再起動に成功しました。"
fi
