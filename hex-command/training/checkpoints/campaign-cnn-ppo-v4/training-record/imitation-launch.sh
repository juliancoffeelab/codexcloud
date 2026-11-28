#!/usr/bin/env bash
set -euo pipefail
source /workspace/.setup/env.sh
cd /workspace/codexcloud/hex-command
run_dir="$PWD/tmp/rl/v2-predictor-queued-pretrain-1h"
py="$PWD/tmp/rl/runtime/venv/bin/python"
export PYTHONPATH="$run_dir/native:$PWD/training"
export PYTHONUNBUFFERED=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
exec 9>"$run_dir/supervisor.lock"
flock -n 9
if [[ ! -f "$run_dir/workload.json" ]]; then
    "$py" "$run_dir/prepare.py" "$run_dir"
fi
if [[ -f "$run_dir/posttune/complete.json" ]]; then
    exit 0
fi
args=()
if [[ -f "$run_dir/posttune/latest.json" ]]; then
    args+=(--resume "$run_dir/posttune")
fi
exec "$py" -m hex_command_rl.mixed_imitation --root "$run_dir" --seconds 3600 --warm-start "$run_dir/v2-policy.pt" --threads 4 "${args[@]}"
