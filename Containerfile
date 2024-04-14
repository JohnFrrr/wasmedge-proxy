FROM scratch
COPY wasmedge_hyper_proxy.wasm /
CMD ["/wasmedge_hyper_proxy.wasm"]