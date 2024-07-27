#!/bin/bash

# ==================================
# リストア・マイグレーション・e2eテスト・負荷試験・採点の順で実施してくれるスクリプト。
# ==================================

(cd webapp && bash ./restore_and_migration.sh)
if [ $? -ne 0 ]; then
    echo -e "採点フローを中断します。"
    exit 1
fi

# e2eテスト
(cd webapp/e2e && bash ./run_e2e_test.sh)
if [ $? -ne 0 ]; then
    echo -e "採点フローを中断します。"
    exit 1
fi

# 負荷試験 & 採点開始
curl localhost:9000/api/group/collect
if [[ $HOSTNAME != app-* ]]; then
    (cd benchmarker && bash ./run_k6_and_score.sh)
    if [ $? -ne 0 ]; then
        echo -e "採点フローを中断します。"
        exit 1
    fi
    exit 0
fi

echo "負荷試験を開始するためのリクエストを送信します。"
COMMIT_ID=$(git rev-parse HEAD)
RESPONSE=$(curl -s -X POST https://stress.ftt2407.dabaas.net/api/queuing_trigger -H "Content-Type: application/json" -d '{"teamId":"'$HOSTNAME'", "commitId":"'$COMMIT_ID'"}')
JOB_ID=$(echo "$RESPONSE" | jq -r '.jobId')
IP_ADDRESS=$(echo "$RESPONSE" | jq -r '.ipAddress')

if [ -z "$JOB_ID" ] || [ -z "$IP_ADDRESS" ]; then
    echo -e "\n\n===================================================\n\n"
    echo -e "負荷試験のリクエストに失敗しました。メンターに報告してください。"
    echo $RESPONSE
    echo -e "\n\n===================================================\n\n"
    exit 1
fi

echo -e "\n\n===================================================\n\n"
echo -e "負荷試験のリクエストに成功しました。"
echo -e "ジョブID: $JOB_ID"
echo -e "負荷試験サーバーIPアドレス: $IP_ADDRESS"
echo -e "上記のジョブIDをもとに負荷試験のステータスを確認できます"
echo -e "bash get_test_status.sh $JOB_ID $IP_ADDRESS"
echo -e "\n\n===================================================\n\n"