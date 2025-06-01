# Note: worker threads (emscripten pthreads) need 
# "Cross-Origin-Embedder-Policy" HTTP header set to "require-corp"
# "Cross-Origin-Opener-Policy" HTTP header set to "same-origin"
simple-http-server --index --coep --coop  web
