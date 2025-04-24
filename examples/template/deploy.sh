#!/bin/bash
start_time=$(date +%H:%M:%S)
start_time_s=$(date +%s)

cargo clippy

cargo test release_all_endpoints -- --ignored
cargo test update_open_api -- --ignored
cargo test release_all_handlers -- --ignored

npx wrangler deploy

end_time=$(date +%H:%M:%S)
end_time_s=$(date +%s)
spend=$(($end_time_s - $start_time_s))
spend_minutes=$(($spend / 60))
echo "✅ $start_time -> $end_time" "Total: $spend seconds ($spend_minutes mins) 🎉🎉🎉"
say "Deploy Successful. Spend: $spend seconds."
echo '\n========= deploy successful. =========\n'
