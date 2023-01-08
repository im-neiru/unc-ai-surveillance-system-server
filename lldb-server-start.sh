
if [ "$LLDB_DEBUG" = "ON" ]; then
    lldb-server platform --listen "*:31166" --server --min-gdbserver-port 31200 --max-gdbserver-port 31300
fi
