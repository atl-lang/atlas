#!/bin/bash
# mode.sh — Mode control (unblock, block-work)

cmd_unblock() {
    sqlite3 "$DB" "UPDATE state SET mode='development', block_work_allowed=1"
    echo "Mode: development | Block work: OK"
}

cmd_block_work() {
    sqlite3 "$DB" "UPDATE state SET mode='hardening', block_work_allowed=0"
    echo "Mode: hardening | Block work: BLOCKED"
}
